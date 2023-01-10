use crate::ast::Visitor;

pub trait AstNode {
    fn accept<K: Visitor>(&self, visitor: &mut K) -> K::Result;
}
