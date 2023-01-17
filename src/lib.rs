#![feature(const_mut_refs)]

#[macro_use]

mod ast;
mod errors;
mod interpreter;
mod parser;
mod scanner;
mod token;
mod tool;

use errors::JBreadErrors;
use parser::Parser;
pub use scanner::*;
pub use token::*;
pub use tool::*;

use scanner::Scanner;
use std::{
    fs::File,
    io::{self, Read},
    sync::Mutex,
};

use crate::interpreter::Interpreter;

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
        let mut interpreter = Interpreter::default();

        file.read_to_string(&mut contents).unwrap();
        self.run(&contents, &mut interpreter);

        if self.has_error {
            std::process::exit(65);
        }
    }

    pub fn run_prompt(&self) {
        let mut interpreter = Interpreter::default();
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            self.run(&input, &mut interpreter);
        }
    }

    pub fn run(&self, source: &str, interpreter: &mut Interpreter) {
        let mut scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner.scan_tokens());
        let ast = parser.parse();

        if let Err(error) = &ast {
            error.report();
            Self::set_error();
            return;
        };

        let ast = ast.unwrap();
        dbg!(scanner.scan_tokens());
        dbg!(&ast);

        let result = interpreter.interpret(&ast);

        if let Err(err) = &result {
            err.report();
            Self::set_error();
            return;
        }
    }

    pub fn error(err: JBreadErrors) {
        Self::report(err);
    }

    pub fn report(error: JBreadErrors) {
        eprintln!("{:?}\n{}", error, error.to_string());
        Self::set_error();
    }
}
