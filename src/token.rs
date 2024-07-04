#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(f32),
    Operator(Operator),
    OpenBracket,
    CloseBracket,
    Variable(String),
    Function(String, Vec<Token>),
    Comma,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    NullishCoalescing,
    Conditional,
    Colon,
    Divide,
    Multiply,
    Add,
    Subtract,
    Not,
}

impl Operator {
    pub fn precidence(&self) -> u8 {
        match self {
            Self::Add | Self::Subtract => 11,
            Self::Multiply | Self::Divide => 12,
            Self::NullishCoalescing => 3,
            Self::Conditional | Self::Colon => 2,
            Self::Not => 14,
        }
    }
}