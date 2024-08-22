use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::interpreter::MolangError;

pub trait MolangEq {
    fn molang_eq(&self, rhs: &Value) -> bool;
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f32),
    String(String),
    Struct(HashMap<String, Value>),
    External(Rc<RefCell<dyn External>>),
    Function(Function),
    Null,
}

impl PartialEq<Value> for Value {
    fn eq(&self, other: &Value) -> bool {
        self.molang_eq(other)
    }
}

impl MolangEq for Value {
    fn molang_eq(&self, rhs: &Value) -> bool {
        match self {
            Value::Number(n) => {
                if let Value::Number(rhs) = rhs {
                    rhs == n
                } else {
                    false
                }
            }

            Value::String(s) => {
                if let Value::String(rhs) = rhs {
                    s == rhs
                } else {
                    false
                }
            }

            Value::Struct(s) => {
                if let Value::Struct(rhs) = rhs {
                    s == rhs
                } else {
                    false
                }
            }

            Value::External(e) => {
                if let Value::External(rhs) = rhs {
                    e.borrow().molang_eq(&Value::External(rhs.clone()))
                } else {
                    false
                }
            }
            Value::Function(f) => {
                if let Value::Function(rhs) = rhs {
                    f == rhs
                } else {
                    false
                }
            }
            Value::Null => {
                if let Value::Null = rhs {
                    true
                } else {
                    false
                }
            }
        }
    }
}

pub trait External: Debug + MolangEq {
    fn get(&mut self, property: &str) -> Value;
    fn set(&mut self, property: &str, value: Value) -> Result<(), MolangError>;
    fn call_function(&mut self, function: &str, args: Vec<Value>) -> Result<Value, MolangError>;

    fn index_get(&mut self, index: Value) -> Result<Value, MolangError>;
    fn index_set(&mut self, index: Value, value: Value) -> Result<(), MolangError>;
}

#[derive(Clone)]
pub struct Function {
    pub f: Rc<RefCell<dyn FnMut(Vec<Value>) -> Result<Value, MolangError>>>,
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Function...")
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

pub trait ToMolangValue {
    fn to_value(self) -> Value;
}

pub trait FromMolangValue {
    fn from_value(v: Value) -> Result<Self, MolangError>
    where
        Self: Sized;
}

impl ToMolangValue for Value {
    fn to_value(self) -> Value {
        self
    }
}

impl<T> ToMolangValue for Option<T>
where
    T: ToMolangValue,
{
    fn to_value(self) -> Value {
        match self {
            Some(s) => s.to_value(),
            None => Value::Null,
        }
    }
}

impl<T> FromMolangValue for Option<T>
where
    T: FromMolangValue,
{
    fn from_value(v: Value) -> Result<Self, MolangError> {
        Ok(match v {
            Value::Null => None,
            a => Some(T::from_value(a)?),
        })
    }
}

impl ToMolangValue for f32 {
    fn to_value(self) -> Value {
        Value::Number(self)
    }
}

impl FromMolangValue for f32 {
    fn from_value(v: Value) -> Result<Self, MolangError> {
        match v {
            Value::Number(n) => Ok(n),
            a => Err(MolangError::TypeError(
                "Number".to_string(),
                format!("{a:?}"),
            )),
        }
    }
}

impl ToMolangValue for HashMap<String, Value> {
    fn to_value(self) -> Value {
        Value::Struct(self)
    }
}
