use crate::{
    ast::{Binary, Grouping, Literal, Stmt, Unary, VisitorExpr, VisitorStmt},
    token::Literal as LiteralEnum,
    AstNode,
};

pub struct AstPrinter {}

impl VisitorExpr for AstPrinter {
    type Result = String;

    fn visit_expr_binary(&mut self, expr: &Binary) -> String {
        self.parenthesize(
            expr.operator.lexeme.as_str(),
            vec![expr.left.clone(), expr.right.clone()],
        )
    }

    fn visit_expr_grouping(&mut self, expr: &Grouping) -> String {
        self.parenthesize("group", vec![expr.expression.clone()])
    }

    fn visit_expr_literal(&mut self, expr: &Literal) -> String {
        if let Some(literal) = &expr.value {
            match literal {
                LiteralEnum::String(s) => s.clone(),
                LiteralEnum::Number(n) => n.to_string(),
                LiteralEnum::Boolean(boolean) => boolean.to_string(),
                LiteralEnum::NaN => "NaN".to_string(),
            }
        } else {
            "nil".to_string()
        }
    }

    fn visit_expr_unary(&mut self, expr: &Unary) -> String {
        self.parenthesize(expr.operator.lexeme.as_str(), vec![expr.right.clone()])
    }
}

impl VisitorStmt for AstPrinter {
    type Result = Stmt;

    fn visit_stmt_expression(&mut self, expr: &crate::ast::Expression) -> Self::Result {
        todo!()
    }

    fn visit_stmt_print(&mut self, expr: &crate::ast::Print) -> Self::Result {
        todo!()
    }
}

impl AstPrinter {
    pub fn print<'b, T: AstNode>(&'b mut self, expr: T) -> String {
        expr.accept(self).into()
    }

    pub fn parenthesize<'b, T: AstNode>(&'b mut self, name: &str, exprs: Vec<Box<T>>) -> String {
        let mut result = String::new();
        result.push_str("(");
        result.push_str(name);
        for expr in exprs.iter() {
            result.push_str(" ");
            result.push_str(expr.accept(self).as_str());
        }
        result.push_str(")");
        result
    }
}

impl Default for AstPrinter {
    fn default() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ast::Expr, Token, TokenTypes};

    #[test]
    fn test_creation() {
        let expr = Expr::Binary(Binary {
            right: Box::new(Expr::Binary(Binary {
                right: Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::Number(2.0)),
                })),
                operator: Token {
                    token_type: TokenTypes::Minus,
                    lexeme: "-".to_string(),
                    literal: None,
                    line: 1,
                },
                left: Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::Number(1.0)),
                })),
            })),
            operator: Token {
                token_type: TokenTypes::Plus,
                lexeme: "+".to_string(),
                literal: None,
                line: 1,
            },
            left: Box::new(Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(2.0)),
            })),
        });
        let mut printer = AstPrinter::default();
        assert_eq!(printer.print(expr), "(+ 2 (- 1 2))");
    }
}
