use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{Expr, Literal, Stmt, VisitorExpr, VisitorStmt},
    errors::{self, JBreadErrors, JBreadResult},
    interpreter::environment::Environment,
    AstNode, AstStmt, Literal as LiteralEnum, Token, TokenTypes,
};

pub struct Interpreter {
    // pub globals: HashMap<String, Value>,
    // pub locals: HashMap<String, Value>,
    pub environment: Rc<RefCell<Environment>>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::default())),
        }
    }
}

impl Interpreter {
    fn new(environment: Rc<RefCell<Environment>>) -> Self {
        Self { environment }
    }

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

    fn execute_block(
        &self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), JBreadErrors> {
        let mut interpreter = Interpreter::new(environment);
        statements
            .iter()
            .try_for_each(|stmt| interpreter.execute(stmt))?;
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
        match self.environment.borrow().get(&expr.name) {
            Ok(value) => Ok(Literal {
                value: value.to_owned(),
            }),
            Err(err) => Err(err),
        }
    }

    fn visit_expr_assign(&mut self, expr: &crate::ast::Assign) -> Self::Result {
        let evaluated = self.evalute(&expr.value)?;
        self.environment
            .borrow_mut()
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

        self.environment
            .borrow_mut()
            .define(&stmt.name.lexeme, expr.value);
        Ok(())
    }

    fn visit_stmt_block(&mut self, expr: &crate::ast::Block) -> Self::Result {
        self.execute_block(
            &expr.statements,
            Rc::new(RefCell::new(Environment::new(self.environment.clone()))),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Interpreter, VisitorExpr, VisitorStmt};
    use crate::{
        ast::{Assign, Binary, Expr, Grouping, Literal, Print, Unary, Var, Variable},
        Literal as LiteralEnum, Token, TokenTypes,
    };

    #[test]
    fn test_binary_str_concat() {
        let expr = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::String("Hello".to_string())),
            })),
            operator: Token::new(TokenTypes::Plus, "+".to_string(), None, 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::String(" World!".to_string())),
            })),
        };
        let mut interpreter = Interpreter::default();

        let parsed_binary_expr = interpreter.visit_expr_binary(&expr);
        assert!(parsed_binary_expr.is_ok());
        assert_eq!(
            parsed_binary_expr.unwrap().value,
            Some(LiteralEnum::String("Hello World!".to_string()))
        );
    }

    #[test]
    fn test_binary_num_add() {
        let expr = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(1.0)),
            })),
            operator: Token::new(TokenTypes::Plus, "+".to_string(), None, 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        };
        let mut interpreter = Interpreter::default();

        let parsed_binary_expr = interpreter.visit_expr_binary(&expr);
        assert!(parsed_binary_expr.is_ok());
        assert_eq!(
            parsed_binary_expr.unwrap().value,
            Some(LiteralEnum::Number(3.0))
        );
    }

    #[test]
    fn test_0_0_division() {
        let expr = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(0.0)),
            })),
            operator: Token::new(TokenTypes::Slash, "/".to_string(), None, 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(0.0)),
            })),
        };
        let mut interpreter = Interpreter::default();

        let parsed_binary_expr = interpreter.visit_expr_binary(&expr);
        assert!(parsed_binary_expr.is_ok());
        assert_eq!(parsed_binary_expr.unwrap().value, Some(LiteralEnum::NaN));
    }

    #[test]
    fn test_binary_multipication() {
        let expr = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
            operator: Token::new(TokenTypes::Star, "*".to_string(), None, 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        };
        let mut interpreter = Interpreter::default();

        let parsed_binary_expr = interpreter.visit_expr_binary(&expr);
        assert!(parsed_binary_expr.is_ok());
        assert_eq!(
            parsed_binary_expr.unwrap().value,
            Some(LiteralEnum::Number(4.0))
        );
    }

    #[test]
    fn test_binary_division() {
        let expr = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(4.0)),
            })),
            operator: Token::new(TokenTypes::Slash, "/".to_string(), None, 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        };
        let mut interpreter = Interpreter::default();

        let parsed_binary_expr = interpreter.visit_expr_binary(&expr);
        assert!(parsed_binary_expr.is_ok());
        assert_eq!(
            parsed_binary_expr.unwrap().value,
            Some(LiteralEnum::Number(2.0))
        );
    }

    #[test]
    fn test_binary_subtraction() {
        let expr = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(4.0)),
            })),
            operator: Token::new(TokenTypes::Minus, "-".to_string(), None, 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        };
        let mut interpreter = Interpreter::default();

        let parsed_binary_expr = interpreter.visit_expr_binary(&expr);
        assert!(parsed_binary_expr.is_ok());
        assert_eq!(
            parsed_binary_expr.unwrap().value,
            Some(LiteralEnum::Number(2.0))
        );
    }

    #[test]
    fn test_binary_greater() {
        let expr = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(4.0)),
            })),
            operator: Token::new(TokenTypes::Greater, ">".to_string(), None, 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        };
        let mut interpreter = Interpreter::default();

        let parsed_binary_expr = interpreter.visit_expr_binary(&expr);
        assert!(parsed_binary_expr.is_ok());
        assert_eq!(
            parsed_binary_expr.unwrap().value,
            Some(LiteralEnum::Boolean(true))
        );
    }

    #[test]
    fn test_binary_greater_equal() {
        let expr = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(4.0)),
            })),
            operator: Token::new(TokenTypes::GreaterEqual, ">=".to_string(), None, 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        };
        let mut interpreter = Interpreter::default();

        let parsed_binary_expr = interpreter.visit_expr_binary(&expr);
        assert!(parsed_binary_expr.is_ok());
        assert_eq!(
            parsed_binary_expr.unwrap().value,
            Some(LiteralEnum::Boolean(true))
        );
    }

    #[test]
    fn test_binary_less() {
        let expr = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(4.0)),
            })),
            operator: Token::new(TokenTypes::Less, "<".to_string(), None, 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        };
        let mut interpreter = Interpreter::default();

        let parsed_binary_expr = interpreter.visit_expr_binary(&expr);
        assert!(parsed_binary_expr.is_ok());
        assert_eq!(
            parsed_binary_expr.unwrap().value,
            Some(LiteralEnum::Boolean(false))
        );
    }

    #[test]
    fn test_unary_negation() {
        let expr = Unary {
            operator: Token::new(TokenTypes::Minus, "-".to_string(), None, 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        };
        let mut interpreter = Interpreter::default();

        let parsed_unary_expr = interpreter.visit_expr_unary(&expr);
        assert!(parsed_unary_expr.is_ok());
        assert_eq!(
            parsed_unary_expr.unwrap().value,
            Some(LiteralEnum::Number(-2.0))
        );
    }

    #[test]
    fn test_grouping() {
        let expr = Grouping {
            expression: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        };
        let mut interpreter = Interpreter::default();

        let parsed_grouping_expr = interpreter.visit_expr_grouping(&expr);
        assert!(parsed_grouping_expr.is_ok());
        assert_eq!(
            parsed_grouping_expr.unwrap().value,
            Some(LiteralEnum::Number(2.0))
        );
    }

    #[test]
    fn test_string_and_int_addition() {
        let expr = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::String("Hello".to_string())),
            })),
            operator: Token::new(TokenTypes::Plus, "+".to_string(), None, 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        };
        let mut interpreter = Interpreter::default();

        let parsed_binary_expr = interpreter.visit_expr_binary(&expr);
        assert!(parsed_binary_expr.is_err());
    }

    #[test]
    fn test_var_fetching_without_initalization() {
        let expr = Variable {
            name: Token::new(TokenTypes::Identifier, "a".to_string(), None, 1),
        };
        let mut interpreter = Interpreter::default();

        let parsed_var_expr = interpreter.visit_expr_variable(&expr);
        assert!(parsed_var_expr.is_err());
    }

    #[test]
    fn test_var_assignment_with_value() {
        let expr = Variable {
            name: Token::new(TokenTypes::Identifier, "a".to_string(), None, 1),
        };
        let mut interpreter = Interpreter::default();
        interpreter
            .environment
            .borrow_mut()
            .define("a", Some(LiteralEnum::Number(2.0)));

        let parsed_var_expr = interpreter.visit_expr_variable(&expr);
        assert!(parsed_var_expr.is_ok());
        assert_eq!(
            parsed_var_expr.unwrap().value,
            Some(LiteralEnum::Number(2.0))
        );
    }

    #[test]
    fn test_var_assignment_with_value_and_assignment() {
        let expr = Variable {
            name: Token::new(TokenTypes::Identifier, "a".to_string(), None, 1),
        };
        let mut interpreter = Interpreter::default();
        interpreter
            .environment
            .borrow_mut()
            .define("a", Some(LiteralEnum::Number(2.0)));

        let parsed_var_expr = interpreter.visit_expr_variable(&expr);
        assert!(parsed_var_expr.is_ok());
        assert_eq!(
            parsed_var_expr.unwrap().value,
            Some(LiteralEnum::Number(2.0))
        );

        let assignment_expr = Assign {
            name: Token::new(TokenTypes::Identifier, "a".to_string(), None, 1),
            value: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(3.0)),
            })),
        };
        let parsed_assignment_expr = interpreter.visit_expr_assign(&assignment_expr);
        assert!(parsed_assignment_expr.is_ok());
        assert_eq!(
            parsed_assignment_expr.unwrap().value,
            Some(LiteralEnum::Number(3.0))
        );
    }

    #[test]
    fn test_print_statement() {
        let expr = Expr::Literal(Literal {
            value: Some(LiteralEnum::Number(2.0)),
        });
        let stmt = Print {
            expression: Box::new(expr),
        };
        let mut interpreter = Interpreter::default();

        let parsed_print_stmt = interpreter.visit_stmt_print(&stmt);
        assert!(parsed_print_stmt.is_ok());
    }

    #[test]
    fn test_var_statement() {
        let stmt = Var {
            name: Token::new(TokenTypes::Identifier, "a".to_string(), None, 1),
            initializer: None,
        };
        let mut interpreter = Interpreter::default();

        let parsed_var_stmt = interpreter.visit_stmt_var(&stmt);
        assert!(parsed_var_stmt.is_ok());
    }
}
