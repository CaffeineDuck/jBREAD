/// Generates the AST for the given grammar.
///
/// # Examples
///
/// ## Example 1
///
/// Code:
/// ```ignore
/// define_ast!(
///     Stmt,
///     ExprStmt { expr: Expr },
///     PrintStmt { expr: Expr },
/// )
/// ```
///
/// Generated Code:
/// ```ignore
/// pub enum Stmt {
///    ExprStmt(ExprStmt),
///    PrintStmt(PrintStmt),
/// }
///
/// pub struct ExprStmt {
///     pub expr: Expr,
/// }
///
/// pub struct PrintStmt {
///     pub expr: Expr,
/// }
/// ```
///
///
/// ## Example 2
/// Code:
/// ```ignore
/// define_ast!(
///     // The name of the generated enum to store all the types.
///     Expr,
///     // The name of the base trait implemented by all the types.
///     AstNode,
///     // The name of the trait implemented by the types for the
///     // corresponding visitor.
///     VisitorExpr,
///     // The name of the associated type in the visitor trait.
///     [
///         // Each variant is a struct of the form `Name { $(field: type)* }`.
///         // The method `accept` is automatically generated for each variant.
///         Binary {
///             left: Box<Expr>,
///             operator: Token,
///             right: Box<Expr>,
///         },
///         // The name method to called by the visitor for this variant.
///         // This method should be implemented by the visitor trait.
///         visit_binary_expr,
///     ],
///     [
///         Unary {
///             operator: Token,
///             right: Box<Expr>,
///         },
///         visit_unary_expr,
///     ]
/// )
/// ```
///
/// Generated Code:
/// ```ignore
/// pub enum Expr {
///     Binary(Binary),
///     Unary(Unary),
/// }
///
/// pub struct Binary {
///     left: Box<Expr>,
///     operator: Token,
///     right: Box<Expr>,
/// }
///
/// impl AstNode for Binary {
///     fn accept<K: VisitorExpr>(&self, visitor: &mut K) -> K::Result {
///         visitor.visit_binary_expr(self)
///     }
/// }
/// //.. and so on for Unary
/// ```
#[macro_export]
macro_rules! define_ast {
    (
        $enum_name:ident,
        $($name:ident {
            $($field:ident: $type:ty),*
        }),+ $(,)?
    ) => {
        #[derive(Debug, Clone)]
        pub enum $enum_name {
            $(
                $name($name),
            )*
        }

        $(
            #[derive(Debug, Clone)]
            pub struct $name {
                $(
                    pub $field: $type,
                )*
            }
        )*
    };

    (
        $trait_name:ident,
        $visitor_trait:ident,
        $enum_name:ident,
        $([
            $name:ident {
                $($field:ident: $type:ty),*
            },
            $method_name:ident],
        )+
    )
    => {
        // Generate the structs
        $(
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            $(pub $field: $type),*
        }
        )*

        // Implement trait for each struct
        $(impl $trait_name for $name {
            fn accept<V: $visitor_trait>(&self, visitor: &mut V) -> V::Result {
                visitor.$method_name(self)
            }
        })*

        // Implement visitor trait
        pub trait $visitor_trait {
            type Result;
            $(fn $method_name(&mut self, expr: &$name) -> Self::Result;)*
        }

        // Create an Enum for the structs
        #[derive(Debug, Clone, PartialEq)]
        pub enum $enum_name {
            $($name($name),)*
        }

        // Implement trait for the enum
        impl $trait_name for $enum_name {
            fn accept<V: $visitor_trait>(&self, visitor: &mut V) -> V::Result {
                match self {
                    $(Self::$name(val) => visitor.$method_name(val),)*
                }
            }
        }
    };
}
