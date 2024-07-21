mod data;
mod interpreter;
mod parser;
mod state;
mod tokeniser;
mod value;

pub use interpreter::run;
pub use interpreter::MolangError;
pub use molang_proc_macro::MolangStruct;
pub use parser::Expr;
use thiserror::Error;
use tokeniser::TokeniseError;
pub use value::FromMolangValue;
pub use value::ToMolangValue;
pub use value::Value;

pub fn compile(expr: &str) -> Result<Expr, CompileError> {
    match tokeniser::tokenise(expr) {
        Err(te) => Err(CompileError::TokeniseError(te)),
        Ok(tokens) => parser::treeify(tokens.as_slice()),
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum CompileError {
    #[error("Tokens before prefix operator")]
    TokensBeforePrefixOperator,

    #[error("Incomplete expression")]
    IncompleteExpression,

    #[error("Tokenise error {0}")]
    TokeniseError(TokeniseError),
}
