mod blockiser;
mod data;
mod interpreter;
mod parser;
mod state;
mod tokeniser;
mod value;

use blockiser::blockise;
use blockiser::Block;
pub use interpreter::MolangError;
pub use molang_proc_macro::MolangStruct;
pub use parser::Expr;
use thiserror::Error;
use tokeniser::TokeniseError;
pub use value::External;
pub use value::FromMolangValue;
pub use value::Function;
pub use value::MolangEq;
pub use value::ToMolangValue;
pub use value::Value;

pub fn compile(expr: &str) -> Result<Block, CompileError> {
    match tokeniser::tokenise(expr) {
        Err(te) => Err(CompileError::TokeniseError(te)),
        Ok(tokens) => blockise(tokens),
    }
}

pub use interpreter::run_block as run;

#[derive(Debug, Error, PartialEq)]
pub enum CompileError {
    #[error("Tokens before prefix operator")]
    TokensBeforePrefixOperator,

    #[error("Incomplete expression")]
    IncompleteExpression,

    #[error("Tokenise error {0}")]
    TokeniseError(TokeniseError),
}
