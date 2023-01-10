#[macro_export]
macro_rules! define_ast {(
    $trait_name:ident,
    $visitor_trait:ident,
    $enum_name:ident,
    $([$name:ident{$($field:ident: $type:ty),*}, $method_name:ident],)+
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
            fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
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
            fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
                match self {
                    $(Self::$name(val) => visitor.$method_name(val),)*
                }
            }
        }
    };
}
