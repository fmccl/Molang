use std::{
    cell::RefCell,
    collections::HashMap,
    io::{BufRead, Write},
    rc::Rc,
};

use molang::{External, Function, MolangEq, MolangError, Value};

#[derive(Debug)]
struct Vector {
    vec: Vec<Value>,
}

impl MolangEq for Vector {
    fn molang_eq(&self, rhs: &Value) -> bool {
        match rhs {
            Value::External(ext) => std::ptr::addr_eq(self, ext.as_ptr()),
            _ => false,
        }
    }
}

impl External for Vector {
    fn get(&mut self, property: &str) -> Value {
        match property {
            "length" => Value::Number(self.vec.len() as f32),
            _ => Value::Null,
        }
    }

    fn call_function(&mut self, function: &str, args: Vec<Value>) -> Result<Value, MolangError> {
        match function {
            "push" => {
                for arg in args {
                    self.vec.push(arg);
                }

                Ok(Value::Null)
            }
            _ => Err(MolangError::FunctionNotFound(function.to_string())),
        }
    }

    fn set(&mut self, property: &str, value: Value) -> Result<(), MolangError> {
        Err(MolangError::NotAssignable(format!("vec.{property}")))
    }

    fn index_get(&mut self, index: Value) -> Result<Value, MolangError> {
        let index = match index {
            Value::Number(n) if can_convert_f32_to_usize(n) => n as usize,
            n => {
                return Err(MolangError::BadAccess(
                    format!("Index {n:?}"),
                    "Vec".to_string(),
                ))
            }
        };
        Ok(self.vec.get(index).unwrap_or(&Value::Null).clone())
    }

    fn index_set<'a>(&'a mut self, index: Value, value: Value) -> Result<(), MolangError> {
        let index = match index {
            Value::Number(n) if can_convert_f32_to_usize(n) => n as usize,
            n => {
                return Err(MolangError::BadAccess(
                    format!("Index {n:?}"),
                    "Vec".to_string(),
                ))
            }
        };

        if self.vec.len() > index {
            Err(MolangError::BadAccess(index.to_string(), "Vec".to_string()))
        } else {
            self.vec[index] = value;

            Ok(())
        }
    }
}

fn can_convert_f32_to_usize(x: f32) -> bool {
    x >= 0.0 && x.is_finite() && x.fract() == 0.0 && x <= usize::MAX as f32
}

fn main() {
    let mut constants = HashMap::new();
    let mut variables = HashMap::new();
    variables.insert("variable".to_string(), Value::Struct(HashMap::new()));
    let mut aliases = HashMap::new();
    aliases.insert("v".to_string(), "variable".to_string());

    constants.insert(
        "array".to_string(),
        Value::Function(Function {
            f: Rc::new(RefCell::new(|args| {
                Ok(Value::External(Rc::new(RefCell::new(Vector { vec: args }))))
            })),
        }),
    );

    println!("fmccl/molang REPL: ");

    loop {
        print!("\x1b[0;36m > ");

        print!("\x1b[0;0m");

        std::io::stdout().flush().unwrap();

        let mut line = "".into();
        let len = std::io::stdin().lock().read_line(&mut line).unwrap();

        let compiled = molang::compile(&line[..len]);

        match compiled {
            Ok(compiled) => {
                println!(
                    "{:?}",
                    molang::run(&compiled, &constants, &mut variables, &aliases)
                );
            }
            Err(error) => {
                println!("{error:?}");
                continue;
            }
        }
    }
}
