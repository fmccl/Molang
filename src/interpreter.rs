use std::{any::TypeId, cell::RefCell, collections::HashMap, error, fmt::{format, Debug}, hash::Hash, rc::Rc};

use thiserror::Error;

use crate::{compile, parser::{AccessExpr, Instruction}, value::Function, Expr, Value};

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

    #[error("Type error: expected `{0}` got `{1}`")]
    TypeError(String, String),

    #[error("Cannot access values of `{1}` by `{0}`")]
    BadAccess(String, String)
}

pub fn run(
    expr: &Expr,
    constants: &HashMap<String, Value>,
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
                    let left = match run(left, constants)? {
                        Value::Number(n) => n,
                        a => return Err(MolangError::TypeError("Number".to_string(), format!("{a:?}")))
                    };
                    let right = match run(right, constants)? {
                        Value::Number(n) => n,
                        a => return Err(MolangError::TypeError("Number".to_string(), format!("{a:?}")))
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
                                if let Value::Null = current {
                                    current = constants.get(name).ok_or(MolangError::VariableNotFound(name.clone()))?.clone();
                                } else if let Value::Struct(struc) = current {
                                    current = struc.get(name).ok_or(MolangError::VariableNotFound(name.clone()))?.clone();
                                } else {
                                    return Err(MolangError::BadAccess(".".to_string(), format!("{current:?}")))
                                }
                            }
                            AccessExpr::Index(_) => todo!(),
                            AccessExpr::Call(args) => {
                                if let Value::Function(function) = current {
                                    let mut v_args = Vec::new();
                                    for arg in args {
                                        v_args.push(run(arg, constants)?)
                                    }
                                    current = function.f.borrow_mut()(v_args)?;
                                } else {
                                    return Err(MolangError::BadAccess("()".to_string(), format!("{current:?}")))
                                }
                            },
                        }
                    }

                    Ok(current)
                }
                Instruction::Conditional(left, right) => {
                    let left = match run(left, constants)? {
                        Value::Number(n) => n,
                        a => return Err(MolangError::TypeError("Number".to_string(), format!("{a:?}")))
                    };

                    let (if_true, if_false) = match right {
                        Expr::Derived(b) => {
                            match b.as_ref() {
                                Instruction::Colon(left, right) => (left, right),
                                _ => return Err(MolangError::SyntaxError("Expected colon to close terenary".to_string())),
                            }
                        },
                        _ => return Err(MolangError::SyntaxError("Expected colon to close terenary".to_string())),
                    };

                    if left == 0.0 {
                        run(if_false, constants)
                    } else {
                        run(if_true, constants)
                    }
                },
                Instruction::NullishCoalescing(left, right) => {
                    match run(left, constants)? {
                        Value::Null => run(right, constants),
                        a => Ok(a)
                    }
                },
                Instruction::Colon(_, _) => Err(MolangError::SyntaxError("Unexpected colon".to_string())),
                Instruction::Not(expr) => {
                    let n = match run(expr, constants)? {
                        Value::Number(n) => n,
                        a => return Err(MolangError::TypeError("Number".to_string(), format!("{a:?}")))
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

#[test]
fn function() {
    
    let mut constants = HashMap::new();

    let mut math = HashMap::new();

    math.insert("max".into(), Value::Function(
        Function { 
            f: Rc::new(RefCell::new(
                |args| {
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
            
                }
            ))
        }
    ));

    constants.insert("math".to_string(), Value::Struct(math));

    assert_eq!(
        Value::Number(500.0),
        run(&compile("math.max(1, 5, 2) * 100").unwrap(), &constants).unwrap()
    );
}

#[test]
fn constant() {
    
    let mut constants = HashMap::new();

    constants.insert("pi".to_string(), Value::Number(3.14));

    assert_eq!(
        Value::Number(3.14*100.),
        run(&compile("pi * 100").unwrap(), &constants).unwrap()
    );
}

#[test]
fn ternary_not() {
    assert_eq!(Value::Number(200.0), run(&compile("!1 ? 100 : 200").unwrap(), &HashMap::new()).unwrap());
}