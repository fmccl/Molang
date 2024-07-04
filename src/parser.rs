use std::collections::VecDeque;

use crate::{
    interpreter::Value,
    token::{Operator, Token},
    tokeniser::tokenise, CompileError,
};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Literal(Value),
    Derived(Box<Instruction>),
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Add(Expr, Expr),
    Subtract(Expr, Expr),
    Multiply(Expr, Expr),
    Divide(Expr, Expr),
    Variable(String),
    FunctionCall(String, Vec<Expr>),
    Conditional(Expr, Expr),
    Colon(Expr, Expr),
    NullishCoalescing(Expr, Expr),
    Not(Expr),
}

pub fn treeify(mut tokens: &[Token]) -> Result<Expr, CompileError> {

    if let [Token::OpenBracket, inner_tokens @ .., Token::CloseBracket] = tokens {
        tokens = inner_tokens
    }

    let mut lowest_precidence_operator_maybe: Option<(usize, &Operator)> = None;

    let mut open_brackets = 0;

    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::OpenBracket => open_brackets += 1,
            Token::CloseBracket => open_brackets -= 1,
            Token::Operator(op) if open_brackets == 0 => {
                if let Some(lowest_precidence_operator) = lowest_precidence_operator_maybe {
                    if (lowest_precidence_operator.1.precidence()) > (op.precidence()) {
                        lowest_precidence_operator_maybe = Some((i, op));
                    }
                } else {
                    lowest_precidence_operator_maybe = Some((i, op));
                }
            }
            _ => {}
        }
    }

    if let Some((i, op)) = lowest_precidence_operator_maybe {
        let left = &tokens[..i];
        let right = &tokens[i + 1..];

        return Ok(Expr::Derived(Box::new(match op {
            Operator::Not => {
                if !left.is_empty() {
                    return Err(CompileError::TokensBeforePrefixOperator)
                }
                Instruction::Not(treeify(right)?)
            },
            Operator::Add => Instruction::Add(treeify(left)?, treeify(right)?),
            Operator::Subtract => Instruction::Subtract(treeify(left)?, treeify(right)?),
            Operator::Multiply => Instruction::Multiply(treeify(left)?, treeify(right)?),
            Operator::Divide => Instruction::Multiply(treeify(left)?, treeify(right)?),
            Operator::Conditional => Instruction::Conditional(treeify(left)?, treeify(right)?),
            Operator::Colon => Instruction::Colon(treeify(left)?, treeify(right)?),
            Operator::NullishCoalescing => Instruction::NullishCoalescing(treeify(left)?, treeify(right)?),
        })));
    } else {
        match tokens {
            [Token::Number(n)] => return Ok(Expr::Literal(Value::Number(*n))),
            [Token::Variable(name)] => {
                return Ok(Expr::Derived(Box::new(Instruction::Variable(name.clone()))))
            }
            [Token::Function(name, args)] => {
                let mut e_args = Vec::new();
                for tokens in comma_split(args) {
                    e_args.push(treeify(tokens)?);
                }
                return Ok(Expr::Derived(Box::new(Instruction::FunctionCall(
                    name.clone(),
                    e_args
                ))))
            }
            a => {
                panic!("Unparsable tokens: {a:?}")
            }
        }
    }
}

fn comma_split<'a>(tokens: &'a Vec<Token>) -> Vec<&'a [Token]> {
    let mut result = Vec::new();
    let mut start = 0;

    for (i, token) in tokens.iter().enumerate() {
        if let Token::Comma = token {
            result.push(&tokens[start..i]);
            start = i + 1;
        }
    }

    result.push(&tokens[start..]);

    if let Some(last) = result.pop() {
        if last.len() != 0 {
            result.push(last);
        }
    }

    result
}

#[test]
fn it_works() {
    assert_eq!(
        Expr::Derived(Box::new(Instruction::FunctionCall(
            "max".to_string(),
            vec![
                Expr::Derived(Box::new(Instruction::Variable("a".to_string()))),
                Expr::Literal(Value::Number(10.0))
            ]
        ))),
        treeify(tokenise("max(a, 10)").as_slice()).unwrap()
    )
}
