use std::{collections::VecDeque, default::Default, fmt::Display};

use thiserror::Error;

use crate::token::{Operator, Token};

enum SequenceAction {
    Advance,
    Done,
    Hold,
}

#[derive(Error, Debug)]
enum TokeniseError {
    Expectation { found: String, expected: String },
}

impl Display for TokeniseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self)).unwrap();
        Ok(())
    }
}

trait State {
    fn handle(
        &mut self,
        c: Option<char>,
    ) -> Result<(Option<Token>, Option<Box<dyn State>>, SequenceAction), TokeniseError>;
}

struct NormalState {}
impl State for NormalState {
    fn handle(
        &mut self,
        c: Option<char>,
    ) -> Result<(Option<Token>, Option<Box<dyn State>>, SequenceAction), TokeniseError> {
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
                Some(Box::new(IdentifierState {
                    ..Default::default()
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
            Some('(') => Ok((Some(Token::OpenBracket), None, SequenceAction::Advance)),
            Some(')') => Ok((Some(Token::CloseBracket), None, SequenceAction::Advance)),

            Some('?') => Ok((None, Some(Box::new(QuestionMarkState{..Default::default()})), SequenceAction::Advance)),

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
impl State for NumberState {
    fn handle(
        &mut self,
        c: Option<char>,
    ) -> Result<(Option<Token>, Option<Box<dyn State>>, SequenceAction), TokeniseError> {
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
struct IdentifierState {
    identifier: String,
    params: Option<String>,
    open_brackets: u8,
}
impl State for IdentifierState {
    fn handle(
        &mut self,
        c: Option<char>,
    ) -> Result<(Option<Token>, Option<Box<dyn State>>, SequenceAction), TokeniseError> {
        if let Some(params) = &mut self.params {
            match c {
                Some('(') => {
                    self.open_brackets += 1;
                    Ok((None, None, SequenceAction::Advance))
                }
                Some(')') if self.open_brackets != 0 => {
                    self.open_brackets -= 1;
                    Ok((None, None, SequenceAction::Advance))
                }
                Some(')') => Ok((
                    Some(Token::Function(self.identifier.clone(), tokenise(&params))),
                    Some(Box::new(NormalState {})),
                    SequenceAction::Advance,
                )),
                None => Err(TokeniseError::Expectation {
                    found: "EOF".to_string(),
                    expected: ")".to_string(),
                }),
                Some(c) => {
                    params.push(c);
                    Ok((None, None, SequenceAction::Advance))
                }
            }
        } else {
            match c {
                Some(c) if c.is_alphanumeric() || c == '_' => {
                    self.identifier.push(c);
                    Ok((None, None, SequenceAction::Advance))
                }
                Some('(') => {
                    self.params = Some(String::new());
                    Ok((None, None, SequenceAction::Advance))
                }
                None => Ok((
                    Some(Token::Variable(self.identifier.clone())),
                    None,
                    SequenceAction::Done,
                )),
                _ => Ok((
                    Some(Token::Variable(self.identifier.clone())),
                    Some(Box::new(NormalState {})),
                    SequenceAction::Hold,
                )),
            }
        }
    }
}

#[derive(Default)]
struct QuestionMarkState {}
impl State for QuestionMarkState {
    fn handle(
            &mut self,
            c: Option<char>,
        ) -> Result<(Option<Token>, Option<Box<dyn State>>, SequenceAction), TokeniseError> {
        match c {
            Some('?') => Ok((Some(Token::Operator(Operator::NullishCoalescing)), Some(Box::new(NormalState{})), SequenceAction::Advance)),
            _ => Ok((Some(Token::Operator(Operator::Conditional)), Some(Box::new(NormalState{})), SequenceAction::Hold))
        }
    }
}

pub fn tokenise(input: &str) -> Vec<Token> {
    let mut state: Box<dyn State> = Box::new(NormalState {});

    let mut i = 0;

    let mut tokens = Vec::new();

    loop {
        let (token, new_state, action) = state.handle(input.chars().nth(i)).unwrap();
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

    tokens
}

#[test]
fn number() {
    assert_eq!(VecDeque::from([Token::Number(100.0)]), tokenise("100.0"));
}

#[test]
fn variable() {
    assert_eq!(VecDeque::from([Token::Number(100.0), Token::Operator(Operator::Multiply), Token::Variable("abc".to_string())]), tokenise("100.0*abc"));
}

#[test]
fn function() {
    assert_eq!(Vec::from([Token::Function("sin".to_string(), vec![Token::Number(1.0)])]), tokenise("sin(1)"));
}

#[test]
fn multiply() {
    assert_eq!(
        VecDeque::from([
            Token::Number(100.0),
            Token::Operator(Operator::Multiply),
            Token::Number(99.0)
        ]),
        tokenise("100.0*99")
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
        tokenise("100.0/99")
    );
}
