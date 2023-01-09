mod scanner;
mod token;

use std::{
    fs::File,
    io::{self, Read},
    sync::Mutex,
};

use scanner::Scanner;

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

pub struct JuniorBread {
    has_error: bool,
}

impl JuniorBread {
    const HAS_ERROR: Mutex<bool> = Mutex::new(false);

    pub fn new() -> Self {
        Self { has_error: false }
    }

    pub fn set_error() {
        *Self::HAS_ERROR.lock().unwrap() = true;
    }

    pub fn remove_error() {
        *Self::HAS_ERROR.lock().unwrap() = false;
    }

    pub fn run_file(&self, path: &str) {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        self.run(&contents);

        if self.has_error {
            std::process::exit(65);
        }
    }

    pub fn run_prompt(&self) {
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            self.run(&input);
        }
    }

    pub fn run(&self, source: &str) {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        for token in tokens {
            println!("{:?}", token);
        }
    }

    pub fn error(line: u32, message: &str) {
        Self::report(line, "", message);
    }

    pub fn report(line: u32, where_: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, where_, message);
        Self::set_error();
    }
}
