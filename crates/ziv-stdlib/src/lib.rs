//! Standard Library registry for Ziv.
//!
//! This crate owns builtin function definitions and categories.

pub mod array;
pub mod container;
pub mod crypto;
pub mod encoding;
pub mod filesystem;
pub mod io;
pub mod math;
pub mod net;
pub mod string;
pub mod utils;

use std::collections::HashMap;

/// Built-in function registry.
pub struct Stdlib {
    functions: HashMap<String, BuiltinFunction>,
}

/// Built-in function signature.
#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub params: Vec<BuiltinParam>,
    pub return_type: Option<String>,
    pub category: String,
    pub description: String,
}

/// Built-in function parameter.
#[derive(Debug, Clone)]
pub struct BuiltinParam {
    pub name: String,
    pub ty: String,
}

impl Stdlib {
    /// Create a new standard library instance with all built-in functions.
    pub fn new() -> Self {
        let mut stdlib = Stdlib {
            functions: HashMap::new(),
        };

        // Core families.
        stdlib.register_io_functions();
        stdlib.register_math_functions();
        stdlib.register_string_functions();
        stdlib.register_array_functions();
        stdlib.register_container_functions();
        stdlib.register_filesystem_functions();
        stdlib.register_net_functions();
        stdlib.register_crypto_functions();
        stdlib.register_encoding_functions();

        // High-frequency utility helpers.
        stdlib.register_utils_functions();

        stdlib
    }

    /// Register a built-in function.
    pub fn register(&mut self, func: BuiltinFunction) {
        self.functions.insert(func.name.clone(), func);
    }

    /// Check whether a function is built-in.
    pub fn is_builtin(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get a built-in function by name.
    pub fn get(&self, name: &str) -> Option<&BuiltinFunction> {
        self.functions.get(name)
    }

    /// Return all built-in functions.
    pub fn all_functions(&self) -> Vec<&BuiltinFunction> {
        self.functions.values().collect()
    }

    /// Return built-ins by category.
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
    use std::collections::HashSet;

    #[test]
    fn test_stdlib_creation() {
        let stdlib = Stdlib::new();
        assert!(stdlib.is_builtin("print"));
        assert!(stdlib.is_builtin("abs"));
        assert!(stdlib.is_builtin("parseInt"));
        assert!(stdlib.is_builtin("readFile"));
        assert!(stdlib.is_builtin("fetch"));
        assert!(stdlib.is_builtin("sha256"));
        assert!(stdlib.is_builtin("base64Encode"));
        assert!(stdlib.is_builtin("vectorNew"));
        assert!(stdlib.is_builtin("hashMapNew"));
    }

    #[test]
    fn test_get_function() {
        let stdlib = Stdlib::new();
        let func = stdlib.get("print");
        assert!(func.is_some());
        assert_eq!(func.expect("print builtin must exist").name, "print");
    }

    #[test]
    fn test_functions_by_category() {
        let stdlib = Stdlib::new();
        let io_funcs = stdlib.functions_by_category("io");
        assert!(!io_funcs.is_empty());

        let utils_funcs = stdlib.functions_by_category("utils");
        assert!(!utils_funcs.is_empty());
        let fs_funcs = stdlib.functions_by_category("filesystem");
        assert!(!fs_funcs.is_empty());
        let net_funcs = stdlib.functions_by_category("net");
        assert!(!net_funcs.is_empty());
        let crypto_funcs = stdlib.functions_by_category("crypto");
        assert!(!crypto_funcs.is_empty());
        let encoding_funcs = stdlib.functions_by_category("encoding");
        assert!(!encoding_funcs.is_empty());
    }

    #[test]
    fn test_all_functions_and_default() {
        let stdlib = Stdlib::default();
        let all = stdlib.all_functions();
        assert!(!all.is_empty());
    }

    #[test]
    fn test_registry_counts_and_categories() {
        let stdlib = Stdlib::new();
        assert_eq!(stdlib.all_functions().len(), 117);
        assert_eq!(stdlib.functions_by_category("io").len(), 9);
        assert_eq!(stdlib.functions_by_category("math").len(), 8);
        assert_eq!(stdlib.functions_by_category("string").len(), 8);
        assert_eq!(stdlib.functions_by_category("array").len(), 8);
        assert_eq!(stdlib.functions_by_category("container").len(), 20);
        assert_eq!(stdlib.functions_by_category("utils").len(), 18);
        assert_eq!(stdlib.functions_by_category("filesystem").len(), 12);
        assert_eq!(stdlib.functions_by_category("net").len(), 10);
        assert_eq!(stdlib.functions_by_category("crypto").len(), 12);
        assert_eq!(stdlib.functions_by_category("encoding").len(), 12);
    }

    #[test]
    fn test_unknown_lookup_and_category_are_empty() {
        let stdlib = Stdlib::new();
        assert!(!stdlib.is_builtin("__missing_builtin__"));
        assert!(stdlib.get("__missing_builtin__").is_none());
        assert!(stdlib
            .functions_by_category("__missing_category__")
            .is_empty());
    }

    #[test]
    fn test_builtin_metadata_is_complete_and_names_unique() {
        let stdlib = Stdlib::new();
        let mut names = HashSet::new();

        for func in stdlib.all_functions() {
            assert!(names.insert(func.name.clone()));
            assert!(!func.category.trim().is_empty());
            assert!(!func.description.trim().is_empty());
        }
    }
}
