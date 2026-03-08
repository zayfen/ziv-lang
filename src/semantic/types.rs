//! Type system for LightLang

use std::fmt;

/// Represents a type in LightLang
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitive types
    Int,
    Float,
    String,
    Bool,
    Void,
    Null,

    // Compound types
    Array(Box<Type>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },

    // Any type (for type inference)
    Any,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Void => write!(f, "void"),
            Type::Null => write!(f, "null"),
            Type::Array(elem) => write!(f, "{}[]", elem),
            Type::Function {
                params,
                return_type,
            } => {
                let params_str: Vec<String> = params.iter().map(|p| format!("{}", p)).collect();
                write!(f, "({}) -> {}", params_str.join(", "), return_type)
            }
            Type::Any => write!(f, "any"),
        }
    }
}

impl From<&str> for Type {
    fn from(s: &str) -> Self {
        match s {
            "int" | "i32" | "i64" => Type::Int,
            "float" | "f32" | "f64" => Type::Float,
            "string" | "str" => Type::String,
            "bool" | "boolean" => Type::Bool,
            "void" => Type::Void,
            "null" => Type::Null,
            "any" => Type::Any,
            _ => Type::Any,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", Type::Int), "int");
        assert_eq!(format!("{}", Type::Float), "float");
        assert_eq!(format!("{}", Type::String), "string");
        assert_eq!(format!("{}", Type::Bool), "bool");
    }

    #[test]
    fn test_array_type() {
        let arr_type = Type::Array(Box::new(Type::Int));
        assert_eq!(format!("{}", arr_type), "int[]");
    }

    #[test]
    fn test_type_from_str_variants() {
        assert_eq!(Type::from("i64"), Type::Int);
        assert_eq!(Type::from("f64"), Type::Float);
        assert_eq!(Type::from("str"), Type::String);
        assert_eq!(Type::from("boolean"), Type::Bool);
        assert_eq!(Type::from("void"), Type::Void);
        assert_eq!(Type::from("null"), Type::Null);
        assert_eq!(Type::from("any"), Type::Any);
        assert_eq!(Type::from("unknown"), Type::Any);
    }

    #[test]
    fn test_display_void_null_any_and_function() {
        assert_eq!(format!("{}", Type::Void), "void");
        assert_eq!(format!("{}", Type::Null), "null");
        assert_eq!(format!("{}", Type::Any), "any");

        let f = Type::Function {
            params: vec![Type::Int, Type::String],
            return_type: Box::new(Type::Bool),
        };
        assert_eq!(format!("{}", f), "(int, string) -> bool");
    }
}
