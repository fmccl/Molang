use std::{
    collections::HashMap,
    io::{BufRead, Write},
};

use molang::Value;

fn main() {
    let constants = HashMap::new();
    let mut variables = HashMap::new();
    variables.insert("variable".to_string(), Value::Struct(HashMap::new()));
    let mut aliases = HashMap::new();
    aliases.insert("v".to_string(), "variable".to_string());

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
