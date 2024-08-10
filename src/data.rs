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
    Assignment,
    Equality,
}

impl Operator {
    pub fn precidence(&self) -> u8 {
        match self {
            Self::Add | Self::Subtract => 11,
            Self::Multiply | Self::Divide => 12,
            Self::NullishCoalescing => 3,
            Self::Conditional | Self::Colon | Self::Assignment => 2,
            Self::Not => 14,
            Self::Equality => 8,
        }
    }
}