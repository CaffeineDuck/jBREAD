use crate::{define_ast, AstNode, AstStmt, Literal as LiteralEnum, Token};

define_ast!(
    AstNode,
    VisitorExpr,
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
    [
        Variable {
            name: Token
        },
        visit_expr_variable
    ],
    [
        Assign {
            name: Token,
            value: Box<Expr>
        },
        visit_expr_assign
    ],
);

define_ast!(
    AstStmt,
    VisitorStmt,
    Stmt,
    [
        Expression {
            expression: Box<Expr>
        },
        visit_stmt_expression
    ],
    [
        Print {
            expression: Box<Expr>
        },
        visit_stmt_print
    ],
    [
        Var {
            name: Token,
            initializer: Option<Box<Expr>>
        },
        visit_stmt_var
    ],
);
