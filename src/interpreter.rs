use std::{any::TypeId, collections::HashMap, error, hash::Hash};

use thiserror::Error;

use crate::{compile, parser::Instruction, Expr};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(f32),
}

#[derive(Error, Debug)]
pub enum MolangError {
    #[error("Function not found: `{0}`")]
    FunctionNotFound(String),

    #[error("Function error: `{0}`")]
    FunctionError(String),

    #[error("Variable not found: `{0}`")]
    VariableNotFound(String),

    #[error("Syntax error: `{0}`")]
    SyntaxError(String)
}

pub fn run(
    expr: &Expr,
    functions: &HashMap<&str, &dyn Fn(Vec<Value>) -> Result<Value, MolangError>>,
    variables: &HashMap<&str, Value>,
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
                    let left = match run(left, functions, variables)? {
                        Value::Number(n) => n,
                    };
                    let right = match run(right, functions, variables)? {
                        Value::Number(n) => n,
                    };
                    Ok(Value::Number(match i {
                        Instruction::Add(_, _) => left + right,
                        Instruction::Subtract(_, _) => left - right,
                        Instruction::Multiply(_, _) => left * right,
                        Instruction::Divide(_, _) => left / right,
                        _ => unreachable!(),
                    }))
                }
                Instruction::FunctionCall(name, args) => {
                    let mut v_args = Vec::new();
                    for arg in args {
                        v_args.push(run(arg, functions, variables)?);
                    }
                    let func = functions.get(name.as_str());
                    match func {
                        Some(func) => func(v_args),
                        None => Err(MolangError::FunctionNotFound(name.clone())),
                    }
                }
                Instruction::Variable(name) => Ok(variables
                    .get(name.as_str())
                    .ok_or(MolangError::VariableNotFound(name.clone()))?
                    .clone()),
                Instruction::Conditional(left, right) => {
                    let left = match run(left, functions, variables)? {
                        Value::Number(n) => n,
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
                        run(if_false, functions, variables)
                    } else {
                        run(if_true, functions, variables)
                    }
                },
                Instruction::NullishCoalescing(_, _) => todo!(),
                Instruction::Colon(_, _) => Err(MolangError::SyntaxError("Unexpected colon".to_string())),
                Instruction::Not(expr) => {
                    let n = match run(expr, functions, variables)? {
                        Value::Number(n) => n,
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
    let mut functions: HashMap<&str, &dyn Fn(Vec<Value>) -> Result<Value, MolangError>> =
        HashMap::new();
    
    functions.insert("max", &|args| {
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

    });

    assert_eq!(
        Value::Number(500.0),
        run(&compile("max(1, 5, 2) * 100").unwrap(), &functions, &HashMap::new()).unwrap()
    );
}

#[test]
fn constant() {
    
    let mut constants = HashMap::new();

    constants.insert("pi", Value::Number(3.14));

    assert_eq!(
        Value::Number(500.0),
        run(&compile("max(1, 5, 2) * 100").unwrap(), &HashMap::new(), &constants).unwrap()
    );
}

#[test]
fn ternary_not() {
    assert_eq!(Value::Number(200.0), run(&compile("!1 ? 100 : 200").unwrap(), &HashMap::new(), &HashMap::new()).unwrap());
}