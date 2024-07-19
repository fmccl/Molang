use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::interpreter::MolangError;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f32),
    Struct(HashMap<String, Value>),
    Function(Function),
    Null,
}

#[derive(Clone)]
pub struct Function {
    pub f: Rc<RefCell<dyn FnMut(Vec<Value>) -> Result<Value, MolangError>>>,
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Function {...}")
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
