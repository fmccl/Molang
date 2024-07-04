
mod tokeniser;
mod parser;
mod token;
mod interpreter;

pub use interpreter::Value;
pub use interpreter::run;
pub use parser::Expr;
use thiserror::Error;

pub fn compile(expr: &str) -> Result<Expr, CompileError> {
    parser::treeify(tokeniser::tokenise(expr).as_slice())
}

#[derive(Debug, Error, PartialEq)]
pub enum CompileError {
    #[error("Tokens before prefix operator")]
    TokensBeforePrefixOperator,
}