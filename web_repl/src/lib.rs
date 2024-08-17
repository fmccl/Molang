use std::collections::HashMap;

use molang::{compile, Value};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct State {
    constants: HashMap<String, Value>,
    variables: HashMap<String, Value>,
    aliases: HashMap<String, String>,
}

#[wasm_bindgen]
pub fn setup() -> State {
    console_error_panic_hook::set_once();

    let mut state = State {
        aliases: HashMap::new(),
        constants: HashMap::new(),
        variables: HashMap::new(),
    };

    state
        .variables
        .insert("variable".into(), Value::Struct(HashMap::new()));

    state.aliases.insert("v".into(), "variable".into());

    state
}

#[wasm_bindgen]
pub fn run(code: &str, state: &mut State) -> String {
    let compiled = compile(code);
    match compiled {
        Ok(block) => {
            match molang::run(
                &block,
                &state.constants,
                &mut state.variables,
                &state.aliases,
            ) {
                Ok(abc) => return format!("{:?}", abc),
                Err(a) => return format!("{:?}", a),
            }
        }
        Err(error) => return format!("{:?}", error),
    }
}
