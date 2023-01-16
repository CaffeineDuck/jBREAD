use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    errors::{Error, JBreadErrors, JBreadResult},
    Literal as LiteralEnum, Token,
};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Option<LiteralEnum>>,
    encolosing: Option<Rc<RefCell<Environment>>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
            encolosing: None,
        }
    }
}

impl Environment {
    pub fn new(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            encolosing: Some(enclosing),
        }
    }

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

    pub fn get(&self, token: &Token) -> JBreadResult<Option<LiteralEnum>> {
        if let Some(value) = self.values.get(&token.lexeme) {
            Ok(value.clone())
        } else if let Some(enclosed) = &self.encolosing {
            Ok(enclosed.to_owned().borrow().get(token)?.to_owned())
        } else {
            Err(self.error(&token))
        }
    }

    pub fn assign(&mut self, name: &Token, value: Option<LiteralEnum>) -> JBreadResult<()> {
        if !self.values.contains_key(name.lexeme.as_str()) {
            if let Some(enclosing) = &mut self.encolosing {
                return enclosing.borrow_mut().assign(name, value);
            } else {
                return Err(self.error(name));
            }
        }
        self.values.insert(name.lexeme.to_string(), value);
        Ok(())
    }
}
