//! Math functions for Ziv standard library

use super::{BuiltinFunction, BuiltinParam, Stdlib};

impl Stdlib {
    /// Register math functions
    pub fn register_math_functions(&mut self) {
        // abs - Absolute value
        self.register(BuiltinFunction {
            name: "abs".to_string(),
            params: vec![BuiltinParam {
                name: "x".to_string(),
                ty: "number".to_string(),
            }],
            return_type: Some("number".to_string()),
            category: "math".to_string(),
            description: "Return the absolute value of a number".to_string(),
        });
        
        // min - Minimum of two numbers
        self.register(BuiltinFunction {
            name: "min".to_string(),
            params: vec![
                BuiltinParam {
                    name: "a".to_string(),
                    ty: "number".to_string(),
                },
                BuiltinParam {
                    name: "b".to_string(),
                    ty: "number".to_string(),
                },
            ],
            return_type: Some("number".to_string()),
            category: "math".to_string(),
            description: "Return the minimum of two numbers".to_string(),
        });
        
        // max - Maximum of two numbers
        self.register(BuiltinFunction {
            name: "max".to_string(),
            params: vec![
                BuiltinParam {
                    name: "a".to_string(),
                    ty: "number".to_string(),
                },
                BuiltinParam {
                    name: "b".to_string(),
                    ty: "number".to_string(),
                },
            ],
            return_type: Some("number".to_string()),
            category: "math".to_string(),
            description: "Return the maximum of two numbers".to_string(),
        });
        
        // sqrt - Square root
        self.register(BuiltinFunction {
            name: "sqrt".to_string(),
            params: vec![BuiltinParam {
                name: "x".to_string(),
                ty: "number".to_string(),
            }],
            return_type: Some("f64".to_string()),
            category: "math".to_string(),
            description: "Return the square root of a number".to_string(),
        });
        
        // pow - Power
        self.register(BuiltinFunction {
            name: "pow".to_string(),
            params: vec![
                BuiltinParam {
                    name: "base".to_string(),
                    ty: "number".to_string(),
                },
                BuiltinParam {
                    name: "exp".to_string(),
                    ty: "number".to_string(),
                },
            ],
            return_type: Some("f64".to_string()),
            category: "math".to_string(),
            description: "Return base raised to the power of exp".to_string(),
        });
        
        // floor - Floor
        self.register(BuiltinFunction {
            name: "floor".to_string(),
            params: vec![BuiltinParam {
                name: "x".to_string(),
                ty: "number".to_string(),
            }],
            return_type: Some("i64".to_string()),
            category: "math".to_string(),
            description: "Return the largest integer less than or equal to x".to_string(),
        });
        
        // ceil - Ceiling
        self.register(BuiltinFunction {
            name: "ceil".to_string(),
            params: vec![BuiltinParam {
                name: "x".to_string(),
                ty: "number".to_string(),
            }],
            return_type: Some("i64".to_string()),
            category: "math".to_string(),
            description: "Return the smallest integer greater than or equal to x".to_string(),
        });
        
        // round - Round
        self.register(BuiltinFunction {
            name: "round".to_string(),
            params: vec![BuiltinParam {
                name: "x".to_string(),
                ty: "number".to_string(),
            }],
            return_type: Some("i64".to_string()),
            category: "math".to_string(),
            description: "Round x to the nearest integer".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_math_functions_registered() {
        let stdlib = Stdlib::new();
        assert!(stdlib.is_builtin("abs"));
        assert!(stdlib.is_builtin("min"));
        assert!(stdlib.is_builtin("max"));
        assert!(stdlib.is_builtin("sqrt"));
        assert!(stdlib.is_builtin("pow"));
        assert!(stdlib.is_builtin("floor"));
        assert!(stdlib.is_builtin("ceil"));
        assert!(stdlib.is_builtin("round"));
    }
    
    #[test]
    fn test_abs_function() {
        let stdlib = Stdlib::new();
        let func = stdlib.get("abs").unwrap();
        assert_eq!(func.name, "abs");
        assert_eq!(func.category, "math");
        assert_eq!(func.params.len(), 1);
    }
    
    #[test]
    fn test_min_max_functions() {
        let stdlib = Stdlib::new();
        let min_func = stdlib.get("min").unwrap();
        assert_eq!(min_func.params.len(), 2);
        
        let max_func = stdlib.get("max").unwrap();
        assert_eq!(max_func.params.len(), 2);
    }
}
