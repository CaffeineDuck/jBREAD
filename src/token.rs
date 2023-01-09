use crate::TokenTypes;
use std::{any::Any, fmt::Debug};

#[derive(Debug)]
pub struct Token {
    token_type: TokenTypes,
    lexeme: String,
    literal: Box<dyn Any>,
    line: u32,
}

impl Token {
    pub fn new<T>(token_type: TokenTypes, lexeme: String, literal: T, line: u32) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            literal: Box::new(literal),
        }
    }
}
