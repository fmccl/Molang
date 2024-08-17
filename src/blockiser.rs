use crate::{parser::treeify, tokeniser::Token, CompileError, Expr};

#[derive(Debug, PartialEq)]
pub struct Block {
    pub multiple: bool,
    pub statements: Vec<Expr>,
}

pub fn blockise(tokens: Vec<Token>) -> Result<Block, CompileError> {
    let mut statements = Vec::new();

    let mut current_start: usize = 0;

    let mut multiple = false;

    for (index, token) in tokens.iter().enumerate() {
        if *token == Token::Semicolon {
            multiple = true;

            statements.push(treeify(&tokens[current_start..index])?);
            current_start = index + 1;
        }
    }

    if !&tokens[current_start..].is_empty() {
        statements.push(treeify(&tokens[current_start..])?);
    }

    if !multiple {
        statements = vec![treeify(&tokens)?];
    }

    Ok(Block {
        multiple,
        statements,
    })
}

mod test {
    use crate::{
        blockiser::{blockise, Block},
        parser::Instruction,
        tokeniser::tokenise,
        Expr, Value,
    };

    #[test]
    fn statements() {
        assert_eq!(
            Block {
                multiple: true,
                statements: vec![
                    Expr::Literal(Value::Number(1.0)),
                    Expr::Literal(Value::Number(1.0))
                ]
            },
            blockise(tokenise("1; 1;").unwrap()).unwrap()
        )
    }

    #[test]
    fn returns() {
        assert_eq!(
            Block {
                multiple: true,
                statements: vec![
                    Expr::Literal(Value::Number(1.0)),
                    Expr::Derived(Box::new(Instruction::Return(Expr::Literal(Value::Number(
                        1.0
                    )))))
                ]
            },
            blockise(tokenise("1; return 1;").unwrap()).unwrap()
        )
    }
}
