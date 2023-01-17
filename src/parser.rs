use crate::{
    ast::{
        Assign, Binary, Block, Expr, Expression, Grouping, Literal, Print, Stmt, Unary, Var,
        Variable,
    },
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
    fn block_statement(&mut self) -> JBreadResult<Stmt>;
    fn statement(&mut self) -> JBreadResult<Stmt>;

    // Actual parsing
    fn parse(&mut self) -> JBreadResult<Vec<Stmt>>;
}

pub trait ParseExpr {
    fn expression(&mut self) -> JBreadResult<Expr>;
    fn assignment(&mut self) -> JBreadResult<Expr>;
    fn equality(&mut self) -> JBreadResult<Expr>;
    fn comparison(&mut self) -> JBreadResult<Expr>;
    fn term(&mut self) -> JBreadResult<Expr>;
    fn factor(&mut self) -> JBreadResult<Expr>;
    fn unary(&mut self) -> JBreadResult<Expr>;
    fn primary(&mut self) -> JBreadResult<Expr>;
}

pub trait ParseStmt {
    fn expression_statement(&mut self) -> JBreadResult<Stmt>;
    fn print_statement(&mut self) -> JBreadResult<Stmt>;
    fn block_statement(&mut self) -> JBreadResult<Stmt>;
    fn statement(&mut self) -> JBreadResult<Stmt>;
}

/// This parser implements the following CFG:
///
/// STATEMENTS:
/// program     → declaration* EOF ;
/// declaration → varDecl | statement ;
/// varDecl     → "var" IDENTIFIER ( "=" expression )? ";" ;
/// statement   → exprStmt | printStmt | block ;
/// exprStmt    → expression ";" ;
/// printStmt   → "print" expression ";" ;
/// block       → "{" declaration* "}" ;
///
/// EXPRESSIONS:
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

    pub fn parse(&mut self) -> JBreadResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    // TODO: Implement error handling while parsing
}

impl<'a> ParseExpr for Parser<'a> {
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
        } else if self.match_token(&[TokenTypes::LeftParen]) {
            let expr = self.expression()?;
            match self.consume(TokenTypes::RightParen, "Expect ')' after expression.") {
                Ok(_) => Ok(Expr::Grouping(Grouping {
                    expression: Box::new(expr),
                })),
                Err(_) => panic!("Error"),
            }
        } else {
            Err(self.error(self.previous(), "Expected Expression"))
        }
    }
}

impl<'a> ParseStmt for Parser<'a> {
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
        if self.match_token(&[TokenTypes::Var]) {
            self.var_decleration()
        } else if self.match_token(&[TokenTypes::Print]) {
            self.print_statement()
        } else if self.match_token(&[TokenTypes::LeftBrace]) {
            self.block_statement()
        } else {
            self.expression_statement()
        }
    }

    fn block_statement(&mut self) -> JBreadResult<Stmt> {
        let mut statements = Vec::new();
        while !self.check(&TokenTypes::RightBrace) && !self.is_at_end() {
            statements.push(self.statement()?);
        }
        self.consume(TokenTypes::RightBrace, "Expect '}' after block.")?;
        Ok(Stmt::Block(Block { statements }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_bool() {
        let tokens = vec![
            Token::new(
                TokenTypes::True,
                "true".to_string(),
                Some(LiteralEnum::Boolean(true)),
                1,
            ),
            Token::new(TokenTypes::Eof, "".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_literal_true = parser.expression();
        assert!(parsed_literal_true.is_ok(), "Failed to parse true literal");
        assert_eq!(
            parsed_literal_true.unwrap(),
            Expr::Literal(crate::ast::Literal {
                value: Some(LiteralEnum::Boolean(true))
            }),
            "Parsed literal bool is not equal to expected literal true"
        );

        let tokens = vec![
            Token::new(
                TokenTypes::False,
                "false".to_string(),
                Some(LiteralEnum::Boolean(false)),
                1,
            ),
            Token::new(TokenTypes::Eof, "".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_literal_false = parser.expression();
        assert!(
            parsed_literal_false.is_ok(),
            "Failed to parse false literal"
        );
        assert_eq!(
            parsed_literal_false.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::Boolean(false))
            }),
            "Parsed literal bool is not equal to expected literal false"
        );
    }

    #[test]
    fn test_literal_nil() {
        let tokens = vec![
            Token::new(TokenTypes::Nil, "nil".to_string(), None, 1),
            Token::new(TokenTypes::Eof, "".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_literal_nil = parser.expression();
        assert!(parsed_literal_nil.is_ok(), "Failed to parse nil literal");
        assert_eq!(
            parsed_literal_nil.unwrap(),
            Expr::Literal(Literal { value: None }),
            "Parsed literal nil is not equal to expected literal nil"
        );
    }

    #[test]
    fn test_literal_nan() {
        let tokens = vec![
            Token::new(TokenTypes::NaN, "nan".to_string(), None, 1),
            Token::new(TokenTypes::Eof, "".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_literal_nan = parser.expression();
        assert!(parsed_literal_nan.is_ok(), "Failed to parse nan literal");
        assert_eq!(
            parsed_literal_nan.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::NaN)
            }),
            "Parsed literal nan is not equal to expected literal nan"
        );
    }

    #[test]
    fn test_literal_string() {
        let tokens = vec![
            Token::new(
                TokenTypes::String,
                "test".to_string(),
                Some(LiteralEnum::String("test".to_string())),
                1,
            ),
            Token::new(TokenTypes::Eof, "".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_literal_string = parser.expression();
        assert!(
            parsed_literal_string.is_ok(),
            "Failed to parse string literal"
        );
        assert_eq!(
            parsed_literal_string.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::String("test".to_string()))
            }),
            "Parsed literal string is not equal to expected literal string"
        );
    }

    #[test]
    fn test_literal_number() {
        let tokens = vec![
            Token::new(
                TokenTypes::Number,
                "1".to_string(),
                Some(LiteralEnum::Number(1.0)),
                1,
            ),
            Token::new(TokenTypes::Eof, "".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_literal_number = parser.expression();
        assert!(
            parsed_literal_number.is_ok(),
            "Failed to parse number literal"
        );
        assert_eq!(
            parsed_literal_number.unwrap(),
            Expr::Literal(Literal {
                value: Some(LiteralEnum::Number(1.0))
            }),
            "Parsed literal number is not equal to expected literal number"
        );
    }

    #[test]
    fn test_unary() {
        let tokens = vec![
            Token::new(TokenTypes::Minus, "-".to_string(), None, 1),
            Token::new(
                TokenTypes::Number,
                "1".to_string(),
                Some(LiteralEnum::Number(1.0)),
                1,
            ),
            Token::new(TokenTypes::Eof, "".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_unary = parser.expression();
        dbg!(&parsed_unary);
        assert!(parsed_unary.is_ok(), "Failed to parse unary");
        assert_eq!(
            parsed_unary.unwrap(),
            Expr::Unary(Unary {
                operator: Token::new(TokenTypes::Minus, "-".to_string(), None, 1),
                right: Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::Number(1.0))
                }))
            }),
            "Parsed unary is not equal to expected unary"
        );
    }

    #[test]
    fn test_grouping() {
        let tokens = vec![
            Token::new(TokenTypes::LeftParen, "(".to_string(), None, 1),
            Token::new(
                TokenTypes::Number,
                "1".to_string(),
                Some(LiteralEnum::Number(1.0)),
                1,
            ),
            Token::new(TokenTypes::RightParen, ")".to_string(), None, 1),
            Token::new(TokenTypes::Eof, "".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_grouping = parser.expression();
        assert!(parsed_grouping.is_ok(), "Failed to parse grouping");
        assert_eq!(
            parsed_grouping.unwrap(),
            Expr::Grouping(Grouping {
                expression: Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::Number(1.0))
                }))
            }),
            "Parsed grouping is not equal to expected grouping"
        );
    }

    #[test]
    fn test_binary() {
        let tokens = vec![
            Token::new(
                TokenTypes::Number,
                "1".to_string(),
                Some(LiteralEnum::Number(1.0)),
                1,
            ),
            Token::new(TokenTypes::Plus, "+".to_string(), None, 1),
            Token::new(
                TokenTypes::Number,
                "1".to_string(),
                Some(LiteralEnum::Number(1.0)),
                1,
            ),
            Token::new(TokenTypes::Eof, "".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_binary = parser.expression();
        assert!(parsed_binary.is_ok(), "Failed to parse binary");
        assert_eq!(
            parsed_binary.unwrap(),
            Expr::Binary(Binary {
                left: Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::Number(1.0))
                })),
                operator: Token::new(TokenTypes::Plus, "+".to_string(), None, 1),
                right: Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::Number(1.0))
                }))
            }),
            "Parsed binary is not equal to expected binary"
        );
    }

    #[test]
    fn test_var_decl() {
        let tokens = vec![
            Token::new(TokenTypes::Var, "var".to_string(), None, 1),
            Token::new(TokenTypes::Identifier, "test".to_string(), None, 1),
            Token::new(TokenTypes::Equal, "=".to_string(), None, 1),
            Token::new(
                TokenTypes::Number,
                "1".to_string(),
                Some(LiteralEnum::Number(1.0)),
                1,
            ),
            Token::new(TokenTypes::Semicolon, ";".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_var_decl = parser.statement();
        dbg!(&parsed_var_decl);

        assert!(parsed_var_decl.is_ok(), "Failed to parse var decl");
        assert_eq!(
            parsed_var_decl.unwrap(),
            Stmt::Var(Var {
                name: Token::new(TokenTypes::Identifier, "test".to_string(), None, 1),
                initializer: Some(Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::Number(1.0))
                })))
            }),
            "Parsed var decl is not equal to expected var decl"
        );
    }

    #[test]
    fn test_var_assign() {
        let tokens = vec![
            Token::new(TokenTypes::Identifier, "test".to_string(), None, 1),
            Token::new(TokenTypes::Equal, "=".to_string(), None, 1),
            Token::new(
                TokenTypes::Number,
                "1".to_string(),
                Some(LiteralEnum::Number(1.0)),
                1,
            ),
            Token::new(TokenTypes::Semicolon, ";".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_var_assign = parser.expression();
        assert!(parsed_var_assign.is_ok(), "Failed to parse var assign");
        assert_eq!(
            parsed_var_assign.unwrap(),
            Expr::Assign(Assign {
                name: Token::new(TokenTypes::Identifier, "test".to_string(), None, 1),
                value: Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::Number(1.0))
                }))
            }),
            "Parsed var assign is not equal to expected var assign"
        );
    }

    #[test]
    fn test_print() {
        let tokens = vec![
            Token::new(TokenTypes::Print, "print".to_string(), None, 1),
            Token::new(
                TokenTypes::Number,
                "1".to_string(),
                Some(LiteralEnum::Number(1.0)),
                1,
            ),
            Token::new(TokenTypes::Semicolon, ";".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_print = parser.statement();
        assert!(parsed_print.is_ok(), "Failed to parse print");
        assert_eq!(
            parsed_print.unwrap(),
            Stmt::Print(Print {
                expression: Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::Number(1.0))
                }))
            }),
            "Parsed print is not equal to expected print"
        );
    }

    #[test]
    fn test_stmt_expression() {
        let tokens = vec![
            Token::new(
                TokenTypes::Number,
                "1".to_string(),
                Some(LiteralEnum::Number(1.0)),
                1,
            ),
            Token::new(TokenTypes::Semicolon, ";".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_stmt_expression = parser.statement();
        assert!(
            parsed_stmt_expression.is_ok(),
            "Failed to parse stmt expression"
        );
        assert_eq!(
            parsed_stmt_expression.unwrap(),
            Stmt::Expression(Expression {
                expression: Box::new(Expr::Literal(Literal {
                    value: Some(LiteralEnum::Number(1.0))
                }))
            }),
            "Parsed stmt expression is not equal to expected stmt expression"
        );
    }

    #[test]
    fn test_block() {
        let tokens = vec![
            Token::new(TokenTypes::LeftBrace, "{".to_string(), None, 1),
            Token::new(TokenTypes::RightBrace, "}".to_string(), None, 1),
        ];
        let mut parser = Parser::new(&tokens);

        let parsed_block = parser.statement();
        dbg!(&parsed_block);

        assert!(parsed_block.is_ok(), "Failed to parse block");
        assert_eq!(
            parsed_block.unwrap(),
            Stmt::Block(Block { statements: vec![] }),
            "Parsed block is not equal to expected block"
        );
    }
}
