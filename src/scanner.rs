use lazy_static::lazy_static;
use std::{collections::HashMap, slice::Iter};

use crate::{
    token::{Literal as LiteralEnum, Token},
    JuniorBread, TokenTypes,
};

#[derive(Debug)]
pub struct Scanner {
    tokens: Vec<Token>,
    source: String,
    start: usize,
    current: usize,
    line: u32,
}

lazy_static! {
    static ref KEYWORDS_MAP: HashMap<&'static str, TokenTypes> = {
        let mut map = HashMap::new();
        map.insert("and", TokenTypes::And);
        map.insert("class", TokenTypes::Class);
        map.insert("else", TokenTypes::Else);
        map.insert("false", TokenTypes::False);
        map.insert("for", TokenTypes::For);
        map.insert("fun", TokenTypes::Fun);
        map.insert("if", TokenTypes::If);
        map.insert("nil", TokenTypes::Nil);
        map.insert("or", TokenTypes::Or);
        map.insert("print", TokenTypes::Print);
        map.insert("return", TokenTypes::Return);
        map.insert("super", TokenTypes::Super);
        map.insert("this", TokenTypes::This);
        map.insert("true", TokenTypes::True);
        map.insert("var", TokenTypes::Var);
        map.insert("while", TokenTypes::While);
        map
    };
}

impl Default for Scanner {
    fn default() -> Self {
        Self {
            tokens: Vec::new(),
            source: String::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Iter<'_, Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_single_token();
        }
        self.tokens
            .push(Token::new(TokenTypes::Eof, "".to_string(), None, self.line));
        self.tokens.iter()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_single_token(&mut self) {
        let chr = self.advance();
        match chr {
            '(' => self.add_token(TokenTypes::LeftParen),
            ')' => self.add_token(TokenTypes::RightParen),
            '{' => self.add_token(TokenTypes::LeftBrace),
            '}' => self.add_token(TokenTypes::RightBrace),
            ',' => self.add_token(TokenTypes::Comma),
            '.' => self.add_token(TokenTypes::Dot),
            '-' => self.add_token(TokenTypes::Minus),
            '+' => self.add_token(TokenTypes::Plus),
            ';' => self.add_token(TokenTypes::Semicolon),
            '*' => self.add_token(TokenTypes::Star),
            '!' => {
                if self.match_next('=') {
                    self.add_token(TokenTypes::BangEqual)
                } else {
                    self.add_token(TokenTypes::Bang)
                }
            }
            '=' => {
                if self.match_next('=') {
                    self.add_token(TokenTypes::EqualEqual)
                } else {
                    self.add_token(TokenTypes::Equal)
                }
            }
            '<' => {
                if self.match_next('=') {
                    self.add_token(TokenTypes::LessEqual)
                } else {
                    self.add_token(TokenTypes::Less)
                }
            }
            '>' => {
                if self.match_next('=') {
                    self.add_token(TokenTypes::GreaterEqual)
                } else {
                    self.add_token(TokenTypes::Greater)
                }
            }
            '/' => {
                if self.match_next('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenTypes::Slash)
                }
            }
            '\t' => (),
            '\r' => (),
            ' ' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            ('0'..='9') => self.number(),
            ('a'..='z') | ('A'..='Z') | '_' => self.identifier(),
            _ => JuniorBread::error(self.line, "Unexpected character."),
        };
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        let text = self.source[self.start..self.current].to_string();
        match KEYWORDS_MAP.get(&text.as_str()).clone() {
            Some(token_type) => self.add_token(token_type.to_owned()),
            None => self.add_token(TokenTypes::Identifier),
        }
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next(1).is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let number = self.source[self.start..self.current].to_string();
        self.add_token_with_value(
            TokenTypes::Number,
            LiteralEnum::Number(number.parse::<f64>().unwrap()),
        );
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            JuniorBread::error(self.line, "Unterminated string.");
            return;
        }
        self.advance();
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token_with_value(TokenTypes::String, LiteralEnum::String(value));
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }

    fn peek_next(&self, count: usize) -> char {
        if self.current + count >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + count).unwrap()
        }
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn advance(&mut self) -> char {
        let chr = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        chr
    }

    fn add_token(&mut self, token_type: TokenTypes) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens
            .push(Token::new(token_type, text, None, self.line));
    }

    fn add_token_with_value(&mut self, token_type: TokenTypes, literal: LiteralEnum) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens
            .push(Token::new(token_type, text, Some(literal), self.line));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_addition() {
        let mut scanner = Scanner::new("1 + 2");
        let tokens = scanner.scan_tokens().collect::<Vec<&Token>>();
        assert_eq!(tokens.len(), 4);
        assert_eq!(
            tokens,
            vec![
                &Token {
                    token_type: TokenTypes::Number,
                    literal: Some(LiteralEnum::Number(1.0)),
                    lexeme: "1".to_string(),
                    line: 1
                },
                &Token {
                    token_type: TokenTypes::Plus,
                    literal: None,
                    lexeme: "+".to_string(),
                    line: 1
                },
                &Token {
                    token_type: TokenTypes::Number,
                    literal: Some(LiteralEnum::Number(2.0)),
                    lexeme: "2".to_string(),
                    line: 1
                },
                &Token {
                    token_type: TokenTypes::Eof,
                    literal: None,
                    lexeme: "".to_string(),
                    line: 1
                }
            ]
        );
    }

    #[test]
    fn test_scanner_comments() {
        let mut scanner = Scanner::new("// This is a comment");
        let tokens: Vec<&Token> = scanner.scan_tokens().collect();
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens,
            vec![&Token {
                token_type: TokenTypes::Eof,
                literal: None,
                lexeme: "".to_string(),
                line: 1
            }]
        );
    }

    #[test]
    fn test_scanner_string() {
        let mut scanner = Scanner::new("\"This is a string\"");
        let tokens: Vec<&Token> = scanner.scan_tokens().collect();
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens,
            vec![
                &Token {
                    token_type: TokenTypes::String,
                    literal: Some(LiteralEnum::String("This is a string".to_string())),
                    lexeme: "\"This is a string\"".to_string(),
                    line: 1
                },
                &Token {
                    token_type: TokenTypes::Eof,
                    literal: None,
                    lexeme: "".to_string(),
                    line: 1
                }
            ]
        );
    }
}
