//! IO functions for Ziv standard library

use super::{BuiltinFunction, BuiltinParam, Stdlib};

impl Stdlib {
    /// Register IO functions
    pub fn register_io_functions(&mut self) {
        // print - Print to stdout without newline
        self.register(BuiltinFunction {
            name: "print".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: None,
            category: "io".to_string(),
            description: "Print a value to stdout without newline".to_string(),
        });
        
        // println - Print to stdout with newline
        self.register(BuiltinFunction {
            name: "println".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: None,
            category: "io".to_string(),
            description: "Print a value to stdout with newline".to_string(),
        });
        
        // read - Read from stdin
        self.register(BuiltinFunction {
            name: "read".to_string(),
            params: vec![],
            return_type: Some("string".to_string()),
            category: "io".to_string(),
            description: "Read a line from stdin".to_string(),
        });
        
        // eprint - Print to stderr without newline
        self.register(BuiltinFunction {
            name: "eprint".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: None,
            category: "io".to_string(),
            description: "Print a value to stderr without newline".to_string(),
        });
        
        // eprintln - Print to stderr with newline
        self.register(BuiltinFunction {
            name: "eprintln".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: None,
            category: "io".to_string(),
            description: "Print a value to stderr with newline".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_io_functions_registered() {
        let stdlib = Stdlib::new();
        assert!(stdlib.is_builtin("print"));
        assert!(stdlib.is_builtin("println"));
        assert!(stdlib.is_builtin("read"));
        assert!(stdlib.is_builtin("eprint"));
        assert!(stdlib.is_builtin("eprintln"));
    }
    
    #[test]
    fn test_print_function() {
        let stdlib = Stdlib::new();
        let func = stdlib.get("print").unwrap();
        assert_eq!(func.name, "print");
        assert_eq!(func.category, "io");
        assert_eq!(func.params.len(), 1);
    }
}
