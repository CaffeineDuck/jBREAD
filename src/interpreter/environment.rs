use std::collections::HashMap;

use crate::{
    errors::{Error, JBreadErrors, JBreadResult},
    Literal as LiteralEnum, Token,
};

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
    fn error(&self, name: &Token) -> JBreadErrors {
        JBreadErrors::RunTimeException(Error::new(
            name.line,
            name.lexeme.clone(),
            "Undefined variable".to_string(),
        ))
    }

    pub fn define(&mut self, name: &str, value: Option<LiteralEnum>) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> JBreadResult<&Option<LiteralEnum>> {
        self.values
            .get(name.lexeme.as_str())
            .ok_or(self.error(name))
    }

    pub fn assign(&mut self, name: &Token, value: Option<LiteralEnum>) -> JBreadResult<()> {
        if !self.values.contains_key(name.lexeme.as_str()) {
            return Err(self.error(name));
        }
        self.values.insert(name.lexeme.to_string(), value);
        Ok(())
    }
}
