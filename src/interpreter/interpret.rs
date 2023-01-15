use crate::{
    ast::{Expr, Literal, Stmt, VisitorExpr, VisitorStmt},
    errors::{self, JBreadErrors, JBreadResult},
    interpreter::environment::Environment,
    AstNode, AstStmt, Literal as LiteralEnum, Token, TokenTypes,
};

pub struct Interpreter {
    // pub globals: HashMap<String, Value>,
    // pub locals: HashMap<String, Value>,
    pub environment: Environment,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self {
            environment: Environment::default(),
        }
    }
}

impl Interpreter {
    fn evalute(&mut self, expr: &Expr) -> JBreadResult<Literal> {
        expr.accept(self)
    }

    fn execute(&mut self, stmt: &Stmt) -> JBreadResult<()> {
        stmt.accept(self)
    }

    fn error(&self, token: &Token, message: &str) -> JBreadErrors {
        JBreadErrors::RunTimeException(errors::Error::new(
            token.line,
            token.lexeme.clone(),
            message.to_string(),
        ))
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> JBreadResult<()> {
        for stmt in stmts.iter() {
            self.execute(stmt)?;
        }
        Ok(())
    }
}

impl VisitorExpr for Interpreter {
    type Result = JBreadResult<Literal>;

    fn visit_expr_binary(&mut self, expr: &crate::ast::Binary) -> Self::Result {
        let left = self
            .evalute(&expr.left)?
            .value
            .ok_or(self.error(&expr.operator, "Left value is not a literal"))?;
        let right = self
            .evalute(&expr.right)?
            .value
            .ok_or(self.error(&expr.operator, "Right value is not a literal"))?;

        let left_num: JBreadResult<f64> = left.clone().try_into();
        let right_num: JBreadResult<f64> = right.clone().try_into();

        let expr = match expr.operator.token_type {
            // For number
            TokenTypes::Minus => LiteralEnum::Number(left_num? - right_num?),
            TokenTypes::Star => LiteralEnum::Number(left_num? * right_num?),
            TokenTypes::Greater => LiteralEnum::Boolean(left_num? > right_num?),
            TokenTypes::GreaterEqual => LiteralEnum::Boolean(left_num? >= right_num?),
            TokenTypes::Less => LiteralEnum::Boolean(left_num? < right_num?),
            TokenTypes::LessEqual => LiteralEnum::Boolean(left_num? <= right_num?),
            // For all types
            TokenTypes::BangEqual => LiteralEnum::Boolean(left != right),
            TokenTypes::EqualEqual => LiteralEnum::Boolean(left == right),
            // For 0/0 division
            TokenTypes::Slash => match (left_num, right_num) {
                (Ok(left), Ok(right)) => {
                    if right == 0.0 && left == 0.0 {
                        LiteralEnum::NaN
                    } else {
                        LiteralEnum::Number(left / right)
                    }
                }
                _ => return Err(self.error(&expr.operator, "Cannot divide non-number")),
            },
            // For addition and string concat
            TokenTypes::Plus => match (left.clone(), right.clone()) {
                (LiteralEnum::String(_), LiteralEnum::String(_)) => {
                    let left_str: String = left.try_into()?;
                    let right_str: String = right.try_into()?;
                    LiteralEnum::String(left_str + &right_str)
                }
                (LiteralEnum::Number(_), LiteralEnum::Number(_)) => {
                    LiteralEnum::Number(left_num? + right_num?)
                }
                _ => return Err(self.error(&expr.operator, "Invalid operands")),
            },

            _ => return Err(self.error(&expr.operator, "Invalid operator for binary expression")),
        };

        Ok(Literal { value: Some(expr) })
    }

    fn visit_expr_grouping(&mut self, expr: &crate::ast::Grouping) -> Self::Result {
        self.evalute(&expr.expression)
    }

    fn visit_expr_literal(&mut self, expr: &crate::ast::Literal) -> Self::Result {
        Ok(expr.to_owned())
    }

    fn visit_expr_unary(&mut self, expr: &crate::ast::Unary) -> Self::Result {
        let right_value = self
            .evalute(&expr.right)?
            .value
            .ok_or(self.error(&expr.operator, "Right value is not a literal"))?;

        let expr = match expr.operator.token_type {
            TokenTypes::Minus => LiteralEnum::Number(-right_value.try_into()?),
            TokenTypes::Bang => LiteralEnum::Boolean(!right_value.try_into()?),
            _ => return Err(self.error(&expr.operator, "Invalid operator for unary expression")),
        };

        Ok(Literal { value: Some(expr) })
    }

    fn visit_expr_variable(&mut self, expr: &crate::ast::Variable) -> Self::Result {
        let value = self.environment.get(&expr.name)?;
        Ok(Literal {
            value: value.to_owned(),
        })
    }

    fn visit_expr_assign(&mut self, expr: &crate::ast::Assign) -> Self::Result {
        let evaluated = self.evalute(&expr.value)?;
        self.environment
            .assign(&expr.name, evaluated.value.clone())?;
        Ok(evaluated)
    }
}

impl VisitorStmt for Interpreter {
    type Result = JBreadResult<()>;

    fn visit_stmt_expression(&mut self, expr: &crate::ast::Expression) -> Self::Result {
        self.evalute(&expr.expression)?;
        Ok(())
    }

    fn visit_stmt_print(&mut self, expr: &crate::ast::Print) -> Self::Result {
        let value = self.evalute(&expr.expression)?;
        println!("{:?}", value);
        Ok(())
    }

    fn visit_stmt_var(&mut self, stmt: &crate::ast::Var) -> Self::Result {
        let expr = match &stmt.initializer {
            Some(expr) => self.evalute(expr)?,
            None => Literal { value: None },
        };

        self.environment.define(&stmt.name.lexeme, expr.value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Binary, Expr, Expression, Grouping, Literal, Print, Stmt, Unary},
        Literal as LiteralEnum, Token, TokenTypes,
    };

    use super::Interpreter;

    #[test]
    fn test_binary_str_concat() {
        // let stmt = Expr::Binary(Binary {
        //     left: Box::new(Expr::Literal(Literal {
        //         value: Some(LiteralEnum::String("Hello ".to_string())),
        //     })),
        //     operator: Token {
        //         token_type: TokenTypes::Plus,
        //         lexeme: "+".to_string(),
        //         literal: None,
        //         line: 1,
        //     },
        //     right: Box::new(Expr::Literal(Literal {
        //         value: Some(LiteralEnum::String("World!".to_string())),
        //     })),
        // });
        let stmt = Stmt::Print(Print {
            expression: Box::new(Expr::Binary(Binary {
                left: Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::String("Hello ".to_string())),
                })),
                operator: Token {
                    token_type: TokenTypes::Plus,
                    lexeme: "+".to_string(),
                    literal: None,
                    line: 1,
                },
                right: Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::String("World!".to_string())),
                })),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&vec![stmt]);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::String("Hello World!".to_string())),
            })
        );
    }

    #[test]
    fn test_binary_num_add() {
        let expr = Expr::Binary(Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(1.0)),
            })),
            operator: Token {
                token_type: TokenTypes::Plus,
                lexeme: "+".to_string(),
                literal: None,
                line: 1,
            },
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&expr);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(3.0)),
            })
        );
    }

    #[test]
    fn test_0_0_division() {
        let expr = Expr::Binary(Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(0.0)),
            })),
            operator: Token {
                token_type: TokenTypes::Slash,
                lexeme: "/".to_string(),
                literal: None,
                line: 1,
            },
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(0.0)),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&expr);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::NaN),
            })
        );
    }

    #[test]
    fn test_binary_multipication() {
        let expr = Expr::Binary(Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
            operator: Token {
                token_type: TokenTypes::Star,
                lexeme: "*".to_string(),
                literal: None,
                line: 1,
            },
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(3.0)),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&expr);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(6.0)),
            })
        );
    }

    #[test]
    fn test_binary_division() {
        let expr = Expr::Binary(Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(6.0)),
            })),
            operator: Token {
                token_type: TokenTypes::Slash,
                lexeme: "/".to_string(),
                literal: None,
                line: 1,
            },
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&expr);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(3.0)),
            })
        );
    }

    #[test]
    fn test_binary_subtraction() {
        let expr = Expr::Binary(Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(6.0)),
            })),
            operator: Token {
                token_type: TokenTypes::Minus,
                lexeme: "-".to_string(),
                literal: None,
                line: 1,
            },
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&expr);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(4.0)),
            })
        );
    }

    #[test]
    fn test_binary_greater() {
        let expr = Expr::Binary(Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(6.0)),
            })),
            operator: Token {
                token_type: TokenTypes::Greater,
                lexeme: ">".to_string(),
                literal: None,
                line: 1,
            },
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&expr);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::Boolean(true)),
            })
        );
    }

    #[test]
    fn test_binary_greater_equal() {
        let expr = Expr::Binary(Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(6.0)),
            })),
            operator: Token {
                token_type: TokenTypes::GreaterEqual,
                lexeme: ">=".to_string(),
                literal: None,
                line: 1,
            },
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&expr);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::Boolean(true)),
            })
        );
    }

    #[test]
    fn test_binary_less() {
        let expr = Expr::Binary(Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(6.0)),
            })),
            operator: Token {
                token_type: TokenTypes::Less,
                lexeme: "<".to_string(),
                literal: None,
                line: 1,
            },
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&expr);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::Boolean(false)),
            })
        );
    }

    #[test]
    fn test_unary_negation() {
        let expr = Expr::Unary(Unary {
            operator: Token::new(TokenTypes::Bang, "!".to_string(), None, 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Boolean(true)),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&expr);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::Boolean(false)),
            })
        );
    }

    #[test]
    fn test_unary_subtraction() {
        let expr = Expr::Unary(Unary {
            operator: Token::new(TokenTypes::Minus, "-".to_string(), None, 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&expr);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(-2.0)),
            })
        );
    }

    #[test]
    fn test_grouping() {
        let expr = Expr::Grouping(Grouping {
            expression: Box::new(Expr::Binary(Binary {
                left: Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::Number(6.0)),
                })),
                operator: Token {
                    token_type: TokenTypes::Minus,
                    lexeme: "-".to_string(),
                    literal: None,
                    line: 1,
                },
                right: Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::Number(2.0)),
                })),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&expr);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(4.0)),
            })
        );
    }

    #[test]
    fn test_string_and_int_addition() {
        let expr = Expr::Binary(Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::String("Hello, ".to_string())),
            })),
            operator: Token {
                token_type: TokenTypes::Plus,
                lexeme: "+".to_string(),
                literal: None,
                line: 1,
            },
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&expr);

        assert!(result.is_err());
    }
}
