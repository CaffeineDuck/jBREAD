use crate::errors::{Error, JBreadErrors};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenTypes {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
}

impl TryInto<f64> for Literal {
    type Error = JBreadErrors;

    fn try_into(self) -> Result<f64, Self::Error> {
        match self {
            Literal::Number(number) => Ok(number),
            _ => Err(JBreadErrors::RunTimeException(Error::new(
                0,
                "Number".to_string(),
                "Cannot convert non-number to number".to_string(),
            ))),
        }
    }
}

impl TryInto<String> for Literal {
    type Error = JBreadErrors;

    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            Literal::String(string) => Ok(string),
            _ => Err(JBreadErrors::RunTimeException(Error::new(
                0,
                "String".to_string(),
                "Cannot convert non-string to string".to_string(),
            ))),
        }
    }
}

impl TryInto<bool> for Literal {
    type Error = JBreadErrors;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Literal::Boolean(boolean) => Ok(boolean),
            _ => Err(JBreadErrors::RunTimeException(Error::new(
                0,
                "Boolean".to_string(),
                "Cannot convert non-boolean to boolean".to_string(),
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenTypes,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: u32,
}

impl Token {
    pub fn new(
        token_type: TokenTypes,
        lexeme: String,
        literal: Option<Literal>,
        line: u32,
    ) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            literal,
        }
    }
}
