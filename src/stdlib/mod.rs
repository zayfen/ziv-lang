//! Standard Library for Ziv
//!
//! This module provides built-in functions and utilities for the Ziv programming language.

pub mod io;
pub mod math;
pub mod string;
pub mod array;

pub use io::*;
pub use math::*;
pub use string::*;
pub use array::*;

use std::collections::HashMap;

/// Built-in function registry
pub struct Stdlib {
    functions: HashMap<String, BuiltinFunction>,
}

/// Built-in function signature
#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub params: Vec<BuiltinParam>,
    pub return_type: Option<String>,
    pub category: String,
    pub description: String,
}

/// Built-in function parameter
#[derive(Debug, Clone)]
pub struct BuiltinParam {
    pub name: String,
    pub ty: String,
}

impl Stdlib {
    /// Create a new standard library instance with all built-in functions
    pub fn new() -> Self {
        let mut stdlib = Stdlib {
            functions: HashMap::new(),
        };
        
        // Register all built-in functions
        stdlib.register_io_functions();
        stdlib.register_math_functions();
        stdlib.register_string_functions();
        stdlib.register_array_functions();
        
        stdlib
    }
    
    /// Register a built-in function
    pub fn register(&mut self, func: BuiltinFunction) {
        self.functions.insert(func.name.clone(), func);
    }
    
    /// Check if a function is a built-in
    pub fn is_builtin(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
    
    /// Get a built-in function by name
    pub fn get(&self, name: &str) -> Option<&BuiltinFunction> {
        self.functions.get(name)
    }
    
    /// Get all built-in functions
    pub fn all_functions(&self) -> Vec<&BuiltinFunction> {
        self.functions.values().collect()
    }
    
    /// Get functions by category
    pub fn functions_by_category(&self, category: &str) -> Vec<&BuiltinFunction> {
        self.functions
            .values()
            .filter(|f| f.category == category)
            .collect()
    }
}

impl Default for Stdlib {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stdlib_creation() {
        let stdlib = Stdlib::new();
        assert!(stdlib.is_builtin("print"));
        assert!(stdlib.is_builtin("abs"));
    }
    
    #[test]
    fn test_get_function() {
        let stdlib = Stdlib::new();
        let func = stdlib.get("print");
        assert!(func.is_some());
        assert_eq!(func.unwrap().name, "print");
    }
    
    #[test]
    fn test_functions_by_category() {
        let stdlib = Stdlib::new();
        let io_funcs = stdlib.functions_by_category("io");
        assert!(!io_funcs.is_empty());
    }
}
