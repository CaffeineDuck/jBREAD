use std::fmt::Debug;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(f64),
}

#[derive(Debug, Clone)]
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
