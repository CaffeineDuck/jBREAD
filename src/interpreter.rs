use crate::{
    ast::{Expr, Literal, Visitor},
    errors::{self, JBreadErrors, JBreadResult},
    AstNode, Literal as LiteralEnum, Token, TokenTypes,
};

pub struct Interpreter {
    // pub globals: HashMap<String, Value>,
    // pub locals: HashMap<String, Value>,
    // pub environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    fn evalute(&mut self, expr: &Expr) -> JBreadResult<Literal> {
        expr.accept(self)
    }

    fn error(&self, token: &Token, message: &str) -> JBreadErrors {
        JBreadErrors::RunTimeException(errors::Error::new(
            token.line,
            message.to_string(),
            token.lexeme.clone(),
        ))
    }

    pub fn interpret(&mut self, expr: &Expr) -> JBreadResult<Expr> {
        let result = self.evalute(expr)?;
        Ok(Expr::Literal(result))
    }
}

impl Visitor for Interpreter {
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
            TokenTypes::Minus => Literal {
                value: Some(LiteralEnum::Number(left_num? - right_num?)),
            },
            TokenTypes::Slash => Literal {
                value: Some(LiteralEnum::Number(left_num? / right_num?)),
            },
            TokenTypes::Star => Literal {
                value: Some(LiteralEnum::Number(left_num? * right_num?)),
            },
            TokenTypes::Greater => Literal {
                value: Some(LiteralEnum::Boolean(left_num? > right_num?)),
            },
            TokenTypes::GreaterEqual => Literal {
                value: Some(LiteralEnum::Boolean(left_num? >= right_num?)),
            },
            TokenTypes::Less => Literal {
                value: Some(LiteralEnum::Boolean(left_num? < right_num?)),
            },
            TokenTypes::LessEqual => Literal {
                value: Some(LiteralEnum::Boolean(left_num? <= right_num?)),
            },
            // For all types
            TokenTypes::BangEqual => Literal {
                value: Some(LiteralEnum::Boolean(left != right)),
            },
            TokenTypes::EqualEqual => Literal {
                value: Some(LiteralEnum::Boolean(left == right)),
            },
            // For addition and string concat
            TokenTypes::Plus => match (left.clone(), right.clone()) {
                (LiteralEnum::String(_), LiteralEnum::String(_)) => {
                    let left_str: String = left.try_into()?;
                    let right_str: String = right.try_into()?;
                    Literal {
                        value: Some(LiteralEnum::String(left_str + &right_str)),
                    }
                }
                (LiteralEnum::Number(_), LiteralEnum::Number(_)) => Literal {
                    value: Some(LiteralEnum::Number(left_num? + right_num?)),
                },
                _ => return Err(self.error(&expr.operator, "Invalid operands")),
            },

            _ => return Err(self.error(&expr.operator, "Invalid operator for binary expression")),
        };

        Ok(expr)
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
            TokenTypes::Minus => Literal {
                value: Some(LiteralEnum::Number(-right_value.try_into()?)),
            },
            TokenTypes::Bang => Literal {
                value: Some(LiteralEnum::Boolean(!right_value.try_into()?)),
            },

            _ => return Err(self.error(&expr.operator, "Invalid operator for unary expression")),
        };

        Ok(expr)
    }
}
