use std::collections::HashMap;

use crate::{Literal as LiteralEnum, Token};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Option<LiteralEnum>>,
}

impl<'a> Default for Environment {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

impl Environment {
    pub fn define(&mut self, name: &str, value: Option<LiteralEnum>) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Option<&Option<LiteralEnum>> {
        self.values.get(name.lexeme.as_str())
    }
}
