use std::{default::Default, fmt::Display};
use thiserror::Error;

use crate::{
    blockiser::{blockise, Block},
    data::Operator,
    state::{SequenceAction, State},
};

#[derive(Debug, PartialEq)]
pub enum Token {
    Number(f32),
    Operator(Operator),
    OpenBracket,
    CloseBracket,
    Access(Vec<Access>),
    Comma,
    Semicolon,
    Block(Block),
}

#[derive(Debug, PartialEq)]
pub enum Access {
    Name(String),
    Index(Vec<Token>),
    Call(Vec<Token>),
}

#[derive(Error, Debug, PartialEq)]
pub enum TokeniseError {
    Expectation { found: String, expected: String },
}

impl Display for TokeniseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self)).unwrap();
        Ok(())
    }
}

struct NormalState {}
impl State<char, Token, TokeniseError> for NormalState {
    fn handle(
        &mut self,
        c: Option<char>,
    ) -> Result<
        (
            Option<Token>,
            Option<Box<dyn State<char, Token, TokeniseError>>>,
            SequenceAction,
        ),
        TokeniseError,
    > {
        match c {
            Some(c) if c.is_numeric() => Ok((
                None,
                Some(Box::new(NumberState {
                    ..Default::default()
                })),
                SequenceAction::Hold,
            )),

            Some(c) if c.is_alphabetic() => Ok((
                None,
                Some(Box::new(AccessTokenState {
                    state: Box::new(IdentifierState {
                        ..Default::default()
                    }),
                    accesses: Vec::new(),
                })),
                SequenceAction::Hold,
            )),

            Some(c) if c.is_whitespace() => Ok((None, None, SequenceAction::Advance)),

            Some(',') => Ok((Some(Token::Comma), None, SequenceAction::Advance)),
            Some('*') => Ok((
                Some(Token::Operator(Operator::Multiply)),
                None,
                SequenceAction::Advance,
            )),
            Some('/') => Ok((
                Some(Token::Operator(Operator::Divide)),
                None,
                SequenceAction::Advance,
            )),
            Some('+') => Ok((
                Some(Token::Operator(Operator::Add)),
                None,
                SequenceAction::Advance,
            )),
            Some(':') => Ok((
                Some(Token::Operator(Operator::Colon)),
                None,
                SequenceAction::Advance,
            )),
            Some('-') => Ok((
                Some(Token::Operator(Operator::Subtract)),
                None,
                SequenceAction::Advance,
            )),
            Some('!') => Ok((
                Some(Token::Operator(Operator::Not)),
                None,
                SequenceAction::Advance,
            )),
            Some(';') => Ok((Some(Token::Semicolon), None, SequenceAction::Advance)),

            Some('(') => Ok((Some(Token::OpenBracket), None, SequenceAction::Advance)),
            Some(')') => Ok((Some(Token::CloseBracket), None, SequenceAction::Advance)),

            Some('?') => Ok((
                None,
                Some(Box::new(DoubleState {
                    target: '?',
                    result_single: Some(Token::Operator(Operator::Conditional)),
                    result_double: Some(Token::Operator(Operator::NullishCoalescing)),
                })),
                SequenceAction::Advance,
            )),

            Some('=') => Ok((
                None,
                Some(Box::new(DoubleState {
                    target: '=',
                    result_single: Some(Token::Operator(Operator::Assignment)),
                    result_double: Some(Token::Operator(Operator::Equality)),
                })),
                SequenceAction::Advance,
            )),

            Some('{') => Ok((
                None,
                Some(Box::new(BlockState {
                    ..Default::default()
                })),
                SequenceAction::Advance,
            )),

            Some(c) => Err(TokeniseError::Expectation {
                found: c.to_string(),
                expected: "anything else".to_string(),
            }),
            None => Ok((None, None, SequenceAction::Done)),
        }
    }
}

#[derive(Default)]
struct NumberState {
    point: bool,
    string: String,
}
impl State<char, Token, TokeniseError> for NumberState {
    fn handle(
        &mut self,
        c: Option<char>,
    ) -> Result<
        (
            Option<Token>,
            Option<Box<dyn State<char, Token, TokeniseError>>>,
            SequenceAction,
        ),
        TokeniseError,
    > {
        match c {
            Some(c) if c.is_numeric() => {
                self.string.push(c);
                Ok((None, None, SequenceAction::Advance))
            }
            Some('.') => {
                if self.point {
                    return Err(TokeniseError::Expectation {
                        found: ".".to_string(),
                        expected: "a digit".to_string(),
                    });
                }
                self.string.push('.');
                self.point = true;
                Ok((None, None, SequenceAction::Advance))
            }
            Some('_') => Ok((None, None, SequenceAction::Advance)),
            None => Ok((
                Some(Token::Number(self.string.parse().unwrap())),
                None,
                SequenceAction::Done,
            )),
            _ => Ok((
                Some(Token::Number(self.string.parse().unwrap())),
                Some(Box::new(NormalState {})),
                SequenceAction::Hold,
            )),
        }
    }
}

#[derive(Default)]
struct BlockState {
    chars: String,
    open: u32,
}
impl State<char, Token, TokeniseError> for BlockState {
    fn handle(
        &mut self,
        c: Option<char>,
    ) -> Result<
        (
            Option<Token>,
            Option<Box<dyn State<char, Token, TokeniseError>>>,
            SequenceAction,
        ),
        TokeniseError,
    > {
        match c {
            Some('}') if self.open == 0 => Ok((
                Some(Token::Block(blockise(tokenise(&self.chars)?).unwrap())),
                Some(Box::new(NormalState {})),
                SequenceAction::Advance,
            )),
            Some('}') => {
                self.chars.push('}');
                self.open -= 1;
                Ok((None, None, SequenceAction::Advance))
            }
            Some('{') => {
                self.chars.push('{');
                self.open += 1;
                Ok((None, None, SequenceAction::Advance))
            }
            Some(c) => {
                self.chars.push(c);
                Ok((None, None, SequenceAction::Advance))
            }
            None => Err(TokeniseError::Expectation {
                found: "EOF".to_string(),
                expected: "}".to_string(),
            }),
        }
    }
}

struct AccessTokenState {
    state: Box<dyn State<char, Access, TokeniseError>>,
    accesses: Vec<Access>,
}
impl State<char, Token, TokeniseError> for AccessTokenState {
    fn handle(
        &mut self,
        c: Option<char>,
    ) -> Result<
        (
            Option<Token>,
            Option<Box<dyn State<char, Token, TokeniseError>>>,
            SequenceAction,
        ),
        TokeniseError,
    > {
        let (access, new_state, action) = self.state.handle(c)?;

        match access {
            Some(access) => self.accesses.push(access),
            None => {}
        }

        match new_state {
            Some(new_state) => {
                self.state = new_state;
            }
            None => {}
        }

        match action {
            SequenceAction::Done => {
                if self.accesses[0] == Access::Name("return".to_string()) {
                    // perf: create this string once
                    return Ok((
                        Some(Token::Operator(Operator::Return)),
                        Some(Box::new(NormalState {})),
                        SequenceAction::Hold,
                    ));
                }
                Ok((
                    Some(Token::Access(std::mem::take(&mut self.accesses))),
                    Some(Box::new(NormalState {})),
                    SequenceAction::Hold,
                ))
            }
            SequenceAction::Advance => Ok((None, None, SequenceAction::Advance)),
            SequenceAction::Hold => Ok((None, None, SequenceAction::Hold)),
        }
    }
}

struct AccessState {}
impl State<char, Access, TokeniseError> for AccessState {
    fn handle(
        &mut self,
        c: Option<char>,
    ) -> Result<
        (
            Option<Access>,
            Option<Box<dyn State<char, Access, TokeniseError>>>,
            SequenceAction,
        ),
        TokeniseError,
    > {
        match c {
            Some(c) if c.is_whitespace() => Ok((None, None, SequenceAction::Advance)),
            Some('.') => Ok((
                None,
                Some(Box::new(IdentifierState {
                    ..Default::default()
                })),
                SequenceAction::Advance,
            )),
            Some('(') => Ok((
                None,
                Some(Box::new(BracketState {
                    call: true,
                    ..Default::default()
                })),
                SequenceAction::Advance,
            )),
            Some('[') => Ok((
                None,
                Some(Box::new(BracketState {
                    call: false,
                    ..Default::default()
                })),
                SequenceAction::Advance,
            )),

            _ => Ok((None, None, SequenceAction::Done)),
        }
    }
}

#[derive(Default)]
struct IdentifierState {
    identifier: String,
}
impl State<char, Access, TokeniseError> for IdentifierState {
    fn handle(
        &mut self,
        c: Option<char>,
    ) -> Result<
        (
            Option<Access>,
            Option<Box<dyn State<char, Access, TokeniseError>>>,
            SequenceAction,
        ),
        TokeniseError,
    > {
        match c {
            Some(c) if c.is_alphanumeric() || c == '_' => {
                self.identifier.push(c);
                Ok((None, None, SequenceAction::Advance))
            }
            _ => Ok((
                Some(Access::Name(self.identifier.clone())),
                Some(Box::new(AccessState {})),
                SequenceAction::Hold,
            )),
        }
    }
}

#[derive(Default)]
struct BracketState {
    call: bool,
    inner: String,
    open_brackets: i32,
}
impl State<char, Access, TokeniseError> for BracketState {
    fn handle(
        &mut self,
        c: Option<char>,
    ) -> Result<
        (
            Option<Access>,
            Option<Box<dyn State<char, Access, TokeniseError>>>,
            SequenceAction,
        ),
        TokeniseError,
    > {
        let open = if self.call { '(' } else { '[' };
        let close = if self.call { ')' } else { ']' };
        match c {
            Some(c) if c == open => {
                self.open_brackets += 1;
                self.inner.push(c);
                Ok((None, None, SequenceAction::Advance))
            }
            Some(c) if c == close && self.open_brackets != 0 => {
                self.open_brackets -= 1;
                self.inner.push(c);
                Ok((None, None, SequenceAction::Advance))
            }
            Some(c) if c == close && self.open_brackets == 0 => {
                self.open_brackets += 1;
                let acc = if self.call {
                    Access::Call(tokenise(&self.inner)?)
                } else {
                    Access::Index(tokenise(&self.inner)?)
                };
                Ok((
                    Some(acc),
                    Some(Box::new(AccessState {})),
                    SequenceAction::Advance,
                ))
            }
            Some(c) => {
                self.inner.push(c);
                Ok((None, None, SequenceAction::Advance))
            }
            None => Err(TokeniseError::Expectation {
                found: "EOF".to_string(),
                expected: ")".to_string(),
            }),
        }
    }
}

struct DoubleState {
    target: char,
    result_single: Option<Token>,
    result_double: Option<Token>,
}
impl State<char, Token, TokeniseError> for DoubleState {
    fn handle(
        &mut self,
        c: Option<char>,
    ) -> Result<
        (
            Option<Token>,
            Option<Box<dyn State<char, Token, TokeniseError>>>,
            SequenceAction,
        ),
        TokeniseError,
    > {
        match c {
            Some(c) if c == self.target => Ok((
                Some(self.result_double.take().unwrap()),
                Some(Box::new(NormalState {})),
                SequenceAction::Advance,
            )),
            _ => Ok((
                Some(self.result_single.take().unwrap()),
                Some(Box::new(NormalState {})),
                SequenceAction::Hold,
            )),
        }
    }
}

pub fn tokenise(input: &str) -> Result<Vec<Token>, TokeniseError> {
    let mut state: Box<dyn State<char, Token, TokeniseError>> = Box::new(NormalState {});

    let mut i = 0;

    let mut tokens = Vec::new();

    loop {
        let (token, new_state, action) = state.handle(input.chars().nth(i))?;
        if let Some(new_state) = new_state {
            state = new_state;
        }
        if let Some(token) = token {
            tokens.push(token);
        }
        match action {
            SequenceAction::Advance => i += 1,
            SequenceAction::Done => break,
            SequenceAction::Hold => {}
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    use crate::{
        data::Operator,
        tokeniser::{tokenise, Access, Token},
    };

    #[test]
    fn number() {
        assert_eq!(
            VecDeque::from([Token::Number(100.0)]),
            tokenise("100.0").unwrap()
        );
    }

    #[test]
    fn function() {
        assert_eq!(
            Vec::from([Token::Access(vec![
                Access::Name("math".to_string()),
                Access::Name("sin".to_string()),
                Access::Call(vec![Token::Number(1.0)])
            ])]),
            tokenise("math.sin(1)").unwrap()
        );
    }

    #[test]
    fn multiply() {
        assert_eq!(
            VecDeque::from([
                Token::Number(100.0),
                Token::Operator(Operator::Multiply),
                Token::Number(99.0)
            ]),
            tokenise("100.0*99").unwrap()
        );
    }

    #[test]
    fn divide() {
        assert_eq!(
            VecDeque::from([
                Token::Number(100.0),
                Token::Operator(Operator::Divide),
                Token::Number(99.0)
            ]),
            tokenise("100.0/99").unwrap()
        );
    }
}
