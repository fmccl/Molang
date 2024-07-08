mod data;
mod interpreter;
mod parser;
mod state;
mod tokeniser;
mod value;

pub use interpreter::run;
use molang_proc_macro::MolangStruct;
pub use parser::Expr;
use thiserror::Error;
pub use value::FromMolangValue;
pub use value::ToMolangValue;
pub use interpreter::MolangError;
pub use value::Value;

pub fn compile(expr: &str) -> Result<Expr, CompileError> {
    parser::treeify(tokeniser::tokenise(expr).as_slice())
}

#[derive(Debug, Error, PartialEq)]
pub enum CompileError {
    #[error("Tokens before prefix operator")]
    TokensBeforePrefixOperator,
}