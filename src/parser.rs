use crate::{
    ast::{Assign, Binary, Expr, Expression, Literal, Print, Stmt, Unary, Var, Variable},
    errors::{Error, JBreadErrors, JBreadResult},
    Literal as LiteralEnum, Token, TokenTypes,
};

pub trait ParseTrait {
    // Expressions parsing
    fn expression(&mut self) -> JBreadResult<Expr>;
    fn assignment(&mut self) -> JBreadResult<Expr>;
    fn equality(&mut self) -> JBreadResult<Expr>;
    fn comparison(&mut self) -> JBreadResult<Expr>;
    fn term(&mut self) -> JBreadResult<Expr>;
    fn factor(&mut self) -> JBreadResult<Expr>;
    fn unary(&mut self) -> JBreadResult<Expr>;
    fn primary(&mut self) -> JBreadResult<Expr>;
    // Statement parsing
    fn expression_statement(&mut self) -> JBreadResult<Stmt>;
    fn print_statement(&mut self) -> JBreadResult<Stmt>;
    fn decleration(&mut self) -> JBreadResult<Stmt>;
    fn statement(&mut self) -> JBreadResult<Stmt>;

    // Actual parsing
    fn parse(&mut self) -> JBreadResult<Vec<Stmt>>;
}

/// This parser implements the following CFG:
/// program     → declaration* EOF ;
/// declaration → varDecl | statement ;
/// varDecl     → "var" IDENTIFIER ( "=" expression )? ";" ;
/// statement   → exprStmt | printStmt ;
/// exprStmt    → expression ";" ;
/// printStmt   → "print" expression ";" ;
/// expression  → equality ;
/// expression  → equality ;
/// equality    → comparison ( ( "!=" | "==" ) comparison )\* ;
/// comparison  → term ( ( ">" | ">=" | "<" | "<=" ) term )\* ;
/// term        → factor ( ( "-" | "+" ) factor )\* ;
/// factor      → unary ( ( "/" | "*" ) unary )\* ;
/// unary       → ( "!" | "-" ) unary | primary ;
/// primary     → NUMBER | STRING | IDENTIFIER | "true" | "false" | "nil" | "(" expression ")" ;
pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn match_token(&mut self, token_types: &[TokenTypes]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn check(&self, token_type: &TokenTypes) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == *token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenTypes::Eof
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn consume(&mut self, token_type: TokenTypes, arg: &str) -> JBreadResult<&Token> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(self.error(self.peek(), arg))
        }
    }

    fn error(&self, peek: &Token, arg: &str) -> JBreadErrors {
        JBreadErrors::ParseError(Error::new(peek.line, peek.lexeme.clone(), arg.to_string()))
    }

    fn var_decleration(&mut self) -> JBreadResult<Stmt> {
        let name = self
            .consume(TokenTypes::Identifier, "Expected a variable name")?
            .to_owned();

        let mut initializer = None;
        if self.match_token(&[TokenTypes::Equal]) {
            initializer = Some(Box::new(self.expression()?));
        }
        self.consume(
            TokenTypes::Semicolon,
            "Expected ';' after variable declaration",
        )?;
        Ok(Stmt::Var(Var { name, initializer }))
    }

    // TODO: Implement error handling while parsing
}

impl<'a> ParseTrait for Parser<'a> {
    fn expression(&mut self) -> JBreadResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, JBreadErrors> {
        let expr = self.equality()?;

        if self.match_token(&[TokenTypes::Equal]) {
            let equals = self.previous().to_owned();
            let value = self.assignment()?;

            match expr {
                Expr::Variable(Variable { name }) => {
                    return Ok(Expr::Assign(Assign {
                        name,
                        value: Box::new(value),
                    }));
                }
                _ => {
                    return Err(self.error(&equals, "Invalid assignment target"));
                }
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> JBreadResult<Expr> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenTypes::BangEqual, TokenTypes::EqualEqual]) {
            let operator = self.previous().to_owned();
            let right = self.comparison()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            })
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> JBreadResult<Expr> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenTypes::Greater,
            TokenTypes::GreaterEqual,
            TokenTypes::Less,
            TokenTypes::LessEqual,
        ]) {
            let operator = self.previous().to_owned();
            let right = self.term()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            })
        }

        Ok(expr)
    }

    fn term(&mut self) -> JBreadResult<Expr> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenTypes::Minus, TokenTypes::Plus]) {
            let operator = self.previous().to_owned();
            let right = self.factor()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            })
        }

        Ok(expr)
    }

    fn factor(&mut self) -> JBreadResult<Expr> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenTypes::Slash, TokenTypes::Star]) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            })
        }

        Ok(expr)
    }

    fn unary(&mut self) -> JBreadResult<Expr> {
        if self.match_token(&[TokenTypes::Bang, TokenTypes::Minus]) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            return Ok(Expr::Unary(Unary {
                right: Box::new(right),
                operator,
            }));
        }

        self.primary()
    }

    fn primary(&mut self) -> JBreadResult<Expr> {
        if self.match_token(&[TokenTypes::False]) {
            Ok(Expr::Literal(Literal {
                value: Some(LiteralEnum::Boolean(false)),
            }))
        } else if self.match_token(&[TokenTypes::True]) {
            Ok(Expr::Literal(Literal {
                value: Some(LiteralEnum::Boolean(true)),
            }))
        } else if self.match_token(&[TokenTypes::Nil]) {
            Ok(Expr::Literal(Literal { value: None }))
        } else if self.match_token(&[TokenTypes::NaN]) {
            Ok(Expr::Literal(Literal {
                value: Some(LiteralEnum::NaN),
            }))
        } else if self.match_token(&[TokenTypes::String, TokenTypes::Number]) {
            Ok(Expr::Literal(Literal {
                value: self.previous().literal.to_owned(),
            }))
        } else if self.match_token(&[TokenTypes::Identifier]) {
            Ok(Expr::Variable(Variable {
                name: self.previous().to_owned(),
            }))
        } else if self.match_token(&[TokenTypes::Var]) {
            Ok(Expr::Variable(Variable {
                name: self.previous().to_owned(),
            }))
        } else if self.match_token(&[TokenTypes::LeftParen]) {
            let expr = self.expression()?;
            match self.consume(TokenTypes::RightParen, "Expect ')' after expression.") {
                Ok(_) => Ok(Expr::Grouping(crate::ast::Grouping {
                    expression: Box::new(expr),
                })),
                Err(_) => panic!("Error"),
            }
        } else {
            Err(self.error(self.previous(), "Expected Expression"))
        }
    }
    fn expression_statement(&mut self) -> JBreadResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenTypes::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(Expression {
            expression: Box::new(expr),
        }))
    }

    fn print_statement(&mut self) -> JBreadResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenTypes::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(Print {
            expression: Box::new(expr),
        }))
    }

    fn statement(&mut self) -> JBreadResult<Stmt> {
        match self.match_token(&[TokenTypes::Print]) {
            true => self.print_statement(),
            false => self.expression_statement(),
        }
    }

    fn parse(&mut self) -> JBreadResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.decleration()?);
        }
        Ok(statements)
    }

    fn decleration(&mut self) -> JBreadResult<Stmt> {
        if self.match_token(&[TokenTypes::Var]) {
            self.var_decleration()
        } else {
            self.statement()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ParseTrait, Parser};
    use crate::{ast::Grouping, Literal, Token, TokenTypes};

    #[test]
    fn test_term() {
        let tokens = vec![
            Token::new(
                TokenTypes::Number,
                "1".to_string(),
                Some(Literal::Number(1.0)),
                1,
            ),
            Token::new(TokenTypes::Plus, "+".to_string(), None, 1),
            Token::new(
                TokenTypes::Number,
                "2".to_string(),
                Some(Literal::Number(2.0)),
                1,
            ),
            Token::new(TokenTypes::Eof, "".to_string(), None, 1),
        ];
        let mut parser = super::Parser::new(&tokens);

        let ast = parser.parse();
        assert!(ast.is_ok());

        let ast = ast.unwrap();
        assert_eq!(
            ast,
            super::Expr::Binary(super::Binary {
                left: Box::new(super::Expr::Literal(super::Literal {
                    value: Some(Literal::Number(1.0))
                })),
                right: Box::new(super::Expr::Literal(super::Literal {
                    value: Some(Literal::Number(2.0))
                })),
                operator: Token::new(TokenTypes::Plus, "+".to_string(), None, 1)
            })
        )
    }

    #[test]
    fn test_unary() {
        let tokens = vec![
            Token::new(TokenTypes::Minus, "-".to_string(), None, 1),
            Token::new(
                TokenTypes::Number,
                "1".to_string(),
                Some(Literal::Number(1.0)),
                1,
            ),
            Token::new(TokenTypes::Eof, "".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let ast = parser.parse();
        assert!(ast.is_ok());

        let ast = ast.unwrap();
        assert_eq!(
            ast,
            super::Expr::Unary(super::Unary {
                right: Box::new(super::Expr::Literal(super::Literal {
                    value: Some(Literal::Number(1.0))
                })),
                operator: Token::new(TokenTypes::Minus, "-".to_string(), None, 1)
            })
        )
    }

    #[test]
    fn test_grouping() {
        let tokens = vec![
            Token::new(TokenTypes::LeftParen, "(".to_string(), None, 1),
            Token::new(
                TokenTypes::Number,
                "1".to_string(),
                Some(Literal::Number(1.0)),
                1,
            ),
            Token::new(TokenTypes::Minus, "-".to_string(), None, 1),
            Token::new(TokenTypes::LeftParen, "(".to_string(), None, 1),
            Token::new(
                TokenTypes::Number,
                "1".to_string(),
                Some(Literal::Number(1.0)),
                1,
            ),
            Token::new(TokenTypes::Plus, "+".to_string(), None, 1),
            Token::new(
                TokenTypes::Number,
                "2".to_string(),
                Some(Literal::Number(2.0)),
                1,
            ),
            Token::new(TokenTypes::RightParen, ")".to_string(), None, 1),
            Token::new(TokenTypes::RightParen, ")".to_string(), None, 1),
            Token::new(TokenTypes::Eof, "".to_string(), None, 1),
        ];

        let mut parser = Parser::new(&tokens);
        let ast = parser.parse();
        assert!(ast.is_ok());

        let ast = ast.unwrap();
        assert_eq!(
            ast,
            super::Expr::Grouping(Grouping {
                expression: Box::new(super::Expr::Binary(super::Binary {
                    left: Box::new(super::Expr::Literal(super::Literal {
                        value: Some(Literal::Number(1.0))
                    })),
                    right: Box::new(super::Expr::Grouping(Grouping {
                        expression: Box::new(super::Expr::Binary(super::Binary {
                            left: Box::new(super::Expr::Literal(super::Literal {
                                value: Some(Literal::Number(1.0))
                            })),
                            right: Box::new(super::Expr::Literal(super::Literal {
                                value: Some(Literal::Number(2.0))
                            })),
                            operator: Token::new(TokenTypes::Plus, "+".to_string(), None, 1)
                        }))
                    })),
                    operator: Token::new(TokenTypes::Minus, "-".to_string(), None, 1)
                }))
            })
        )
    }
}
