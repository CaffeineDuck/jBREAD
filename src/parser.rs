use crate::{
    ast::{Binary, Expr, Literal, Unary},
    errors::{Error, JBreadErrors, JBreadResult},
    Token, TokenTypes,
};

pub trait ParseTrait {
    fn expression(&mut self) -> JBreadResult<Expr>;
    fn equality(&mut self) -> JBreadResult<Expr>;
    fn comparison(&mut self) -> JBreadResult<Expr>;
    fn term(&mut self) -> JBreadResult<Expr>;
    fn unary(&mut self) -> JBreadResult<Expr>;
    fn primary(&mut self) -> JBreadResult<Expr>;
    fn parse(&mut self) -> JBreadResult<Expr>;
}

/// This parser implements the following CFG:
/// expression     → equality ;
/// equality       → comparison ( ( "!=" | "==" ) comparison )\* ;
/// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )\* ;
/// term           → factor ( ( "-" | "+" ) factor )\* ;
/// factor         → unary ( ( "/" | "*" ) unary )\* ;
/// unary          → ( "!" | "-" ) unary | primary ;
/// primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
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

    fn consume(&self, right_paren: TokenTypes, arg: &str) -> JBreadResult<()> {
        if self.check(&right_paren) {
            return Ok(());
        }
        Err(self.error(self.peek(), arg))
    }

    fn error(&self, peek: &Token, arg: &str) -> JBreadErrors {
        JBreadErrors::ParseError(Error::new(peek.line, peek.lexeme.clone(), arg.to_string()))
    }

    // TODO: Implement error handling while parsing
}

impl<'a> ParseTrait for Parser<'a> {
    fn expression(&mut self) -> JBreadResult<Expr> {
        self.equality()
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
        let mut expr = self.unary()?;

        while self.match_token(&[TokenTypes::Minus, TokenTypes::Plus]) {
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
            dbg!(&right, &operator);
            return Ok(Expr::Unary(Unary {
                right: Box::new(right),
                operator,
            }));
        }

        self.primary()
    }

    fn primary(&mut self) -> JBreadResult<Expr> {
        if self.match_token(&[
            TokenTypes::False,
            TokenTypes::True,
            TokenTypes::Nil,
            TokenTypes::String,
            TokenTypes::Number,
        ]) {
            Ok(Expr::Literal(Literal {
                value: self.previous().literal.to_owned(),
            }))
        } else if self.match_token(&[TokenTypes::LeftParen]) {
            let expr = match self.expression()? {
                Expr::Grouping(grouping) => grouping,
                _ => panic!("Expected grouping"),
            };
            match self.consume(TokenTypes::RightParen, "Expect ')' after expression.") {
                Ok(_) => Ok(Expr::Grouping(expr)),
                Err(_) => panic!("Error"),
            }
        } else {
            Err(self.error(self.peek(), "Expected Expression"))
        }
    }

    fn parse(&mut self) -> JBreadResult<Expr> {
        self.expression()
    }
}
