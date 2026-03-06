//! Array functions for Ziv standard library

use super::{BuiltinFunction, BuiltinParam, Stdlib};

impl Stdlib {
    /// Register array functions
    pub fn register_array_functions(&mut self) {
        // push - Add element to array
        self.register(BuiltinFunction {
            name: "push".to_string(),
            params: vec![
                BuiltinParam {
                    name: "arr".to_string(),
                    ty: "array".to_string(),
                },
                BuiltinParam {
                    name: "element".to_string(),
                    ty: "any".to_string(),
                },
            ],
            return_type: Some("array".to_string()),
            category: "array".to_string(),
            description: "Add an element to the end of an array".to_string(),
        });
        
        // pop - Remove last element
        self.register(BuiltinFunction {
            name: "pop".to_string(),
            params: vec![BuiltinParam {
                name: "arr".to_string(),
                ty: "array".to_string(),
            }],
            return_type: Some("any".to_string()),
            category: "array".to_string(),
            description: "Remove and return the last element of an array".to_string(),
        });
        
        // len - Array length
        self.register(BuiltinFunction {
            name: "arrlen".to_string(),
            params: vec![BuiltinParam {
                name: "arr".to_string(),
                ty: "array".to_string(),
            }],
            return_type: Some("i64".to_string()),
            category: "array".to_string(),
            description: "Return the length of an array".to_string(),
        });
        
        // get - Get element at index
        self.register(BuiltinFunction {
            name: "get".to_string(),
            params: vec![
                BuiltinParam {
                    name: "arr".to_string(),
                    ty: "array".to_string(),
                },
                BuiltinParam {
                    name: "index".to_string(),
                    ty: "i64".to_string(),
                },
            ],
            return_type: Some("any".to_string()),
            category: "array".to_string(),
            description: "Get element at specified index".to_string(),
        });
        
        // set - Set element at index
        self.register(BuiltinFunction {
            name: "set".to_string(),
            params: vec![
                BuiltinParam {
                    name: "arr".to_string(),
                    ty: "array".to_string(),
                },
                BuiltinParam {
                    name: "index".to_string(),
                    ty: "i64".to_string(),
                },
                BuiltinParam {
                    name: "value".to_string(),
                    ty: "any".to_string(),
                },
            ],
            return_type: Some("array".to_string()),
            category: "array".to_string(),
            description: "Set element at specified index".to_string(),
        });
        
        // first - Get first element
        self.register(BuiltinFunction {
            name: "first".to_string(),
            params: vec![BuiltinParam {
                name: "arr".to_string(),
                ty: "array".to_string(),
            }],
            return_type: Some("any".to_string()),
            category: "array".to_string(),
            description: "Get the first element of an array".to_string(),
        });
        
        // last - Get last element
        self.register(BuiltinFunction {
            name: "last".to_string(),
            params: vec![BuiltinParam {
                name: "arr".to_string(),
                ty: "array".to_string(),
            }],
            return_type: Some("any".to_string()),
            category: "array".to_string(),
            description: "Get the last element of an array".to_string(),
        });
        
        // reverse - Reverse array
        self.register(BuiltinFunction {
            name: "reverse".to_string(),
            params: vec![BuiltinParam {
                name: "arr".to_string(),
                ty: "array".to_string(),
            }],
            return_type: Some("array".to_string()),
            category: "array".to_string(),
            description: "Reverse the order of elements in an array".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_array_functions_registered() {
        let stdlib = Stdlib::new();
        assert!(stdlib.is_builtin("push"));
        assert!(stdlib.is_builtin("pop"));
        assert!(stdlib.is_builtin("arrlen"));
        assert!(stdlib.is_builtin("get"));
        assert!(stdlib.is_builtin("set"));
        assert!(stdlib.is_builtin("first"));
        assert!(stdlib.is_builtin("last"));
        assert!(stdlib.is_builtin("reverse"));
    }
    
    #[test]
    fn test_push_function() {
        let stdlib = Stdlib::new();
        let func = stdlib.get("push").unwrap();
        assert_eq!(func.name, "push");
        assert_eq!(func.category, "array");
        assert_eq!(func.params.len(), 2);
    }
    
    #[test]
    fn test_array_len_function() {
        let stdlib = Stdlib::new();
        let func = stdlib.get("arrlen").unwrap();
        assert_eq!(func.params.len(), 1);
    }
}
