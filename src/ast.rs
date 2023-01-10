use crate::{define_ast, AstNode, Literal as LiteralEnum, Token};

define_ast!(
    AstNode,
    Visitor,
    Expr,
    [
        Binary {
            left: Box<Expr>,
            operator: Token,
            right: Box<Expr>
        },
        visit_expr_binary
    ],
    [
        Grouping {
            expression: Box<Expr>
        },
        visit_expr_grouping
    ],
    [
        Literal {
            value: Option<LiteralEnum>
        },
        visit_expr_literal
    ],
    [
        Unary {
            operator: Token,
            right: Box<Expr>
        },
        visit_expr_unary
    ],
);
