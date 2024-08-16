use crate::{
    data::Operator,
    tokeniser::{Access, Token},
    CompileError, Value,
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
    Access(Vec<AccessExpr>),
    Conditional(Expr, Expr),
    Colon(Expr, Expr),
    NullishCoalescing(Expr, Expr),
    Not(Expr),
    Equality(Expr, Expr),
    Assignment(Expr, Expr),
}

#[derive(Debug, PartialEq)]
pub enum AccessExpr {
    Name(String),
    Index(Expr),
    Call(Vec<Expr>),
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

        Ok(Expr::Derived(Box::new(match op {
            Operator::Not => {
                if !left.is_empty() {
                    return Err(CompileError::TokensBeforePrefixOperator);
                }
                Instruction::Not(treeify(right)?)
            }
            Operator::Equality => Instruction::Equality(treeify(left)?, treeify(right)?),
            Operator::Assignment => Instruction::Assignment(treeify(left)?, treeify(right)?),
            Operator::Add => Instruction::Add(treeify(left)?, treeify(right)?),
            Operator::Subtract => Instruction::Subtract(treeify(left)?, treeify(right)?),
            Operator::Multiply => Instruction::Multiply(treeify(left)?, treeify(right)?),
            Operator::Divide => Instruction::Divide(treeify(left)?, treeify(right)?),
            Operator::Conditional => Instruction::Conditional(treeify(left)?, treeify(right)?),
            Operator::Colon => Instruction::Colon(treeify(left)?, treeify(right)?),
            Operator::NullishCoalescing => {
                Instruction::NullishCoalescing(treeify(left)?, treeify(right)?)
            }
        })))
    } else {
        match tokens {
            [Token::Number(n)] => Ok(Expr::Literal(Value::Number(*n))),
            [Token::Access(accesses)] => {
                let mut access_exprs = Vec::new();
                for access in accesses {
                    match access {
                        Access::Call(all_args_tokens) => {
                            let mut args: Vec<Expr> = Vec::new();
                            let args_tokens = comma_split(all_args_tokens);
                            for arg_tokens in args_tokens {
                                args.push(treeify(arg_tokens)?);
                            }
                            access_exprs.push(AccessExpr::Call(args));
                        }
                        Access::Name(name) => access_exprs.push(AccessExpr::Name(name.clone())),
                        Access::Index(tokens) => {
                            access_exprs.push(AccessExpr::Index(treeify(tokens)?))
                        }
                    }
                }
                Ok(Expr::Derived(Box::new(Instruction::Access(access_exprs))))
            }
            _ => Err(CompileError::IncompleteExpression),
        }
    }
}

fn comma_split(tokens: &[Token]) -> Vec<&[Token]> {
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
        if !last.is_empty() {
            result.push(last);
        }
    }

    result
}
