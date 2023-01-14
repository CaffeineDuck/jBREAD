use crate::ast::{VisitorExpr, VisitorStmt};

pub trait AstStmt {
    fn accept<V: VisitorStmt>(&self, visitor: &mut V) -> V::Result;
}

pub trait AstNode {
    fn accept<K: VisitorExpr>(&self, visitor: &mut K) -> K::Result;
}
