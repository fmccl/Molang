use std::{collections::HashMap, fmt::Debug};

use thiserror::Error;

use crate::{
    parser::{AccessExpr, Instruction},
    Expr, Value,
};

#[derive(Error, Debug)]
pub enum MolangError {
    #[error("Function not found: `{0}`")]
    FunctionNotFound(String),

    #[error("Function error: `{0}`")]
    FunctionError(String),

    #[error("Variable not found: `{0}`")]
    VariableNotFound(String),

    #[error("Syntax error: `{0}`")]
    SyntaxError(String),

    #[error("Not assignable: `{0}`")]
    NotAssignable(String),

    #[error("Type error: expected `{0}` got `{1}`")]
    TypeError(String, String),

    #[error("Cannot access values of `{1}` by `{0}`")]
    BadAccess(String, String),
}

pub fn run(
    expr: &Expr,
    constants: &HashMap<String, Value>,
    variables: &mut HashMap<String, Value>,
    aliases: &HashMap<String, String>,
) -> Result<Value, MolangError> {
    match expr {
        Expr::Literal(expr) => Ok(expr.clone()),
        Expr::Derived(i) => {
            let i = i.as_ref();
            match i {
                Instruction::Add(left, right)
                | Instruction::Subtract(left, right)
                | Instruction::Multiply(left, right)
                | Instruction::Divide(left, right) => {
                    let left = match run(left, constants, variables, aliases)? {
                        Value::Number(n) => n,
                        a => {
                            return Err(MolangError::TypeError(
                                "Number".to_string(),
                                format!("{a:?}"),
                            ))
                        }
                    };
                    let right = match run(right, constants, variables, aliases)? {
                        Value::Number(n) => n,
                        a => {
                            return Err(MolangError::TypeError(
                                "Number".to_string(),
                                format!("{a:?}"),
                            ))
                        }
                    };
                    Ok(Value::Number(match i {
                        Instruction::Add(_, _) => left + right,
                        Instruction::Subtract(_, _) => left - right,
                        Instruction::Multiply(_, _) => left * right,
                        Instruction::Divide(_, _) => left / right,
                        _ => unreachable!(),
                    }))
                }
                Instruction::Access(accesses) => {
                    let mut current = Value::Null;

                    for access in accesses {
                        match access {
                            AccessExpr::Name(name) => {
                                let mut name = name;
                                if let Value::Null = current {
                                    if let Some(alias) = aliases.get(name) {
                                        name = alias;
                                    }

                                    current = constants
                                        .get(name)
                                        .or(variables.get(name))
                                        .ok_or(MolangError::VariableNotFound(name.clone()))?
                                        .clone();
                                } else if let Value::Struct(struc) = current {
                                    current = struc
                                        .get(name)
                                        .ok_or(MolangError::VariableNotFound(name.clone()))?
                                        .clone();
                                } else {
                                    return Err(MolangError::BadAccess(
                                        ".".to_string(),
                                        format!("{current:?}"),
                                    ));
                                }
                            }
                            AccessExpr::Index(_) => todo!(),
                            AccessExpr::Call(args) => {
                                if let Value::Function(function) = current {
                                    let mut v_args = Vec::new();
                                    for arg in args {
                                        v_args.push(run(arg, constants, variables, aliases)?)
                                    }
                                    current = function.f.borrow_mut()(v_args)?;
                                } else {
                                    return Err(MolangError::BadAccess(
                                        "()".to_string(),
                                        format!("{current:?}"),
                                    ));
                                }
                            }
                        }
                    }

                    Ok(current)
                }
                Instruction::Assignment(left, right) => {
                    let accesses: &Vec<AccessExpr>;

                    match left {
                        Expr::Literal(_) => {
                            return Err(MolangError::NotAssignable(format!("{left:?}")))
                        }
                        Expr::Derived(instruction) => match instruction.as_ref() {
                            Instruction::Access(a) => {
                                accesses = a;
                            }
                            _ => return Err(MolangError::NotAssignable(format!("{left:?}"))),
                        },
                    }

                    let mut current: *mut Value = &mut Value::Null;

                    for access in accesses {
                        match access {
                            AccessExpr::Name(name) => {
                                let mut name = name;
                                if let Value::Null = unsafe { current.as_ref().unwrap() } {
                                    loop {
                                        if let Some(long_name) = aliases.get(name) {
                                            name = long_name;
                                        }
                                        if let Some(some_current) = variables.get_mut(name) {
                                            current = some_current;
                                            break;
                                        } else {
                                            if constants.contains_key(name) {
                                                return Err(MolangError::NotAssignable(format!(
                                                    "Constant {name}"
                                                )));
                                            } else {
                                                return Err(MolangError::VariableNotFound(
                                                    format!("{name}"),
                                                ));
                                            }
                                        }
                                    }
                                } else if let Value::Struct(struc) =
                                    unsafe { current.as_mut().unwrap() }
                                {
                                    let l_current = struc.get_mut(name);
                                    if let Some(l_current) = l_current {
                                        current = l_current;
                                    } else {
                                        struc.insert(name.clone(), Value::Struct(HashMap::new()));
                                        current = struc.get_mut(name).unwrap();
                                    }
                                } else {
                                    return Err(MolangError::BadAccess(
                                        ".".to_string(),
                                        format!("{current:?}"),
                                    ));
                                }
                            }
                            AccessExpr::Index(_) => todo!(),
                            AccessExpr::Call(_) => {
                                return Err(MolangError::NotAssignable(format!("{access:?}")));
                            }
                        }
                    }

                    unsafe { *current = run(right, constants, variables, aliases)? };

                    Ok(unsafe { (*current).clone() })
                }
                Instruction::Eqaulity(left, right) => Ok(Value::Number(
                    (run(left, constants, variables, aliases)?
                        == run(right, constants, variables, aliases)?)
                    .into(),
                )),
                Instruction::Conditional(left, right) => {
                    let left = match run(left, constants, variables, aliases)? {
                        Value::Number(n) => n,
                        a => {
                            return Err(MolangError::TypeError(
                                "Number".to_string(),
                                format!("{a:?}"),
                            ))
                        }
                    };

                    let (if_true, if_false) = match right {
                        Expr::Derived(b) => match b.as_ref() {
                            Instruction::Colon(left, right) => (left, right),
                            _ => {
                                return Err(MolangError::SyntaxError(
                                    "Expected colon to close terenary".to_string(),
                                ))
                            }
                        },
                        _ => {
                            return Err(MolangError::SyntaxError(
                                "Expected colon to close terenary".to_string(),
                            ))
                        }
                    };

                    if left == 0.0 {
                        run(if_false, constants, variables, aliases)
                    } else {
                        run(if_true, constants, variables, aliases)
                    }
                }
                Instruction::NullishCoalescing(left, right) => {
                    match run(left, constants, variables, aliases)? {
                        Value::Null => run(right, constants, variables, aliases),
                        a => Ok(a),
                    }
                }
                Instruction::Colon(_, _) => {
                    Err(MolangError::SyntaxError("Unexpected colon".to_string()))
                }
                Instruction::Not(expr) => {
                    let n = match run(expr, constants, variables, aliases)? {
                        Value::Number(n) => n,
                        a => {
                            return Err(MolangError::TypeError(
                                "Number".to_string(),
                                format!("{a:?}"),
                            ))
                        }
                    };
                    if n == 0.0 {
                        Ok(Value::Number(1.0))
                    } else {
                        Ok(Value::Number(0.0))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use crate::{compile, run, value::Function, MolangError, Value};

    #[test]
    fn function() {
        let mut constants = HashMap::new();

        let mut math = HashMap::new();

        math.insert(
            "max".into(),
            Value::Function(Function {
                f: Rc::new(RefCell::new(|args| {
                    let mut biggest: Option<f32> = None;

                    for arg in args {
                        if let Value::Number(num) = arg {
                            match biggest {
                                None => biggest = Some(num),
                                Some(big) if num > big => biggest = Some(num),
                                _ => {}
                            }
                        } else {
                            return Err(MolangError::FunctionError("Expected a number".into()));
                        }
                    }

                    Ok(Value::Number(biggest.ok_or(MolangError::FunctionError(
                        "No arguments passed to max".into(),
                    ))?))
                })),
            }),
        );

        constants.insert("math".to_string(), Value::Struct(math));

        assert_eq!(
            Value::Number(500.0),
            run(
                &compile("math.max(1, 5, 2) * 100").unwrap(),
                &constants,
                &mut HashMap::new(),
                &mut HashMap::new(),
            )
            .unwrap()
        );
    }

    #[test]
    fn constant() {
        let mut constants = HashMap::new();

        constants.insert("pi".to_string(), Value::Number(3.14));

        assert_eq!(
            Value::Number(3.14 * 100.),
            run(
                &compile("pi * 100").unwrap(),
                &constants,
                &mut HashMap::new(),
                &mut HashMap::new(),
            )
            .unwrap()
        );
    }

    #[test]
    fn ternary_not() {
        assert_eq!(
            Value::Number(200.0),
            run(
                &compile("!1 ? 100 : 200").unwrap(),
                &HashMap::new(),
                &mut HashMap::new(),
                &mut HashMap::new(),
            )
            .unwrap()
        );
    }

    #[test]
    fn assignment() {
        let variables = &mut HashMap::new();
        variables.insert("lolz".to_string(), Value::Struct(HashMap::new()));
        assert_eq!(
            Value::Number(200.0),
            run(
                &compile("lolz.nested.property = 200").unwrap(),
                &HashMap::new(),
                variables,
                &HashMap::new()
            )
            .unwrap()
        );
        assert_eq!(
            Value::Number(200.0),
            run(
                &compile("lolz.nested.property").unwrap(),
                &HashMap::new(),
                variables,
                &HashMap::new()
            )
            .unwrap()
        );
    }
}
