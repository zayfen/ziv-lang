//! Utility high-frequency builtin functions.

use super::{BuiltinFunction, BuiltinParam, Stdlib};

impl Stdlib {
    /// Register utility helper builtins.
    pub fn register_utils_functions(&mut self) {
        self.register(BuiltinFunction {
            name: "parseInt".to_string(),
            params: vec![
                BuiltinParam {
                    name: "text".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "radix".to_string(),
                    ty: "i64".to_string(),
                },
            ],
            return_type: Some("i64".to_string()),
            category: "utils".to_string(),
            description: "Parse an integer from text with optional radix".to_string(),
        });

        self.register(BuiltinFunction {
            name: "parseFloat".to_string(),
            params: vec![BuiltinParam {
                name: "text".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("f64".to_string()),
            category: "utils".to_string(),
            description: "Parse a floating-point number from text".to_string(),
        });

        self.register(BuiltinFunction {
            name: "isNaN".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: Some("bool".to_string()),
            category: "utils".to_string(),
            description: "Return whether value is NaN-like".to_string(),
        });

        self.register(BuiltinFunction {
            name: "isFinite".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: Some("bool".to_string()),
            category: "utils".to_string(),
            description: "Return whether value is finite".to_string(),
        });

        self.register(BuiltinFunction {
            name: "Number".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: Some("number".to_string()),
            category: "utils".to_string(),
            description: "Coerce value to number".to_string(),
        });

        self.register(BuiltinFunction {
            name: "String".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "utils".to_string(),
            description: "Coerce value to string".to_string(),
        });

        self.register(BuiltinFunction {
            name: "Boolean".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: Some("bool".to_string()),
            category: "utils".to_string(),
            description: "Coerce value to boolean".to_string(),
        });

        self.register(BuiltinFunction {
            name: "jsonParse".to_string(),
            params: vec![BuiltinParam {
                name: "text".to_string(),
                ty: "string".to_string(),
            }],
            return_type: Some("any".to_string()),
            category: "utils".to_string(),
            description: "Parse JSON text".to_string(),
        });

        self.register(BuiltinFunction {
            name: "jsonStringify".to_string(),
            params: vec![BuiltinParam {
                name: "value".to_string(),
                ty: "any".to_string(),
            }],
            return_type: Some("string".to_string()),
            category: "utils".to_string(),
            description: "Serialize value to JSON".to_string(),
        });

        self.register(BuiltinFunction {
            name: "includes".to_string(),
            params: vec![
                BuiltinParam {
                    name: "text".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "search".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("bool".to_string()),
            category: "utils".to_string(),
            description: "Check whether text includes search".to_string(),
        });

        self.register(BuiltinFunction {
            name: "indexOf".to_string(),
            params: vec![
                BuiltinParam {
                    name: "text".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "search".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("i64".to_string()),
            category: "utils".to_string(),
            description: "Return index of search in text".to_string(),
        });

        self.register(BuiltinFunction {
            name: "startsWith".to_string(),
            params: vec![
                BuiltinParam {
                    name: "text".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "prefix".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("bool".to_string()),
            category: "utils".to_string(),
            description: "Return whether text starts with prefix".to_string(),
        });

        self.register(BuiltinFunction {
            name: "endsWith".to_string(),
            params: vec![
                BuiltinParam {
                    name: "text".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "suffix".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("bool".to_string()),
            category: "utils".to_string(),
            description: "Return whether text ends with suffix".to_string(),
        });

        self.register(BuiltinFunction {
            name: "split".to_string(),
            params: vec![
                BuiltinParam {
                    name: "text".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "sep".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("array".to_string()),
            category: "utils".to_string(),
            description: "Split text by separator".to_string(),
        });

        self.register(BuiltinFunction {
            name: "replace".to_string(),
            params: vec![
                BuiltinParam {
                    name: "text".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "pattern".to_string(),
                    ty: "string".to_string(),
                },
                BuiltinParam {
                    name: "replacement".to_string(),
                    ty: "string".to_string(),
                },
            ],
            return_type: Some("string".to_string()),
            category: "utils".to_string(),
            description: "Replace first pattern in text".to_string(),
        });

        self.register(BuiltinFunction {
            name: "map".to_string(),
            params: vec![
                BuiltinParam {
                    name: "arr".to_string(),
                    ty: "array".to_string(),
                },
                BuiltinParam {
                    name: "fn".to_string(),
                    ty: "function".to_string(),
                },
            ],
            return_type: Some("array".to_string()),
            category: "utils".to_string(),
            description: "Map array with callback".to_string(),
        });

        self.register(BuiltinFunction {
            name: "filter".to_string(),
            params: vec![
                BuiltinParam {
                    name: "arr".to_string(),
                    ty: "array".to_string(),
                },
                BuiltinParam {
                    name: "fn".to_string(),
                    ty: "function".to_string(),
                },
            ],
            return_type: Some("array".to_string()),
            category: "utils".to_string(),
            description: "Filter array with callback".to_string(),
        });

        self.register(BuiltinFunction {
            name: "reduce".to_string(),
            params: vec![
                BuiltinParam {
                    name: "arr".to_string(),
                    ty: "array".to_string(),
                },
                BuiltinParam {
                    name: "fn".to_string(),
                    ty: "function".to_string(),
                },
                BuiltinParam {
                    name: "initial".to_string(),
                    ty: "any".to_string(),
                },
            ],
            return_type: Some("any".to_string()),
            category: "utils".to_string(),
            description: "Reduce array with callback and initial value".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_param(func: &BuiltinFunction, idx: usize, name: &str, ty: &str) {
        assert_eq!(func.params[idx].name, name);
        assert_eq!(func.params[idx].ty, ty);
    }

    #[test]
    fn test_utils_functions_registered() {
        let stdlib = Stdlib::new();
        for name in [
            "parseInt",
            "parseFloat",
            "isNaN",
            "isFinite",
            "Number",
            "String",
            "Boolean",
            "jsonParse",
            "jsonStringify",
            "includes",
            "indexOf",
            "startsWith",
            "endsWith",
            "split",
            "replace",
            "map",
            "filter",
            "reduce",
        ] {
            assert!(stdlib.is_builtin(name), "missing builtin: {name}");
        }
    }

    #[test]
    fn test_utils_function_signatures() {
        let stdlib = Stdlib::new();

        let parse_int = stdlib.get("parseInt").expect("parseInt must exist");
        assert_eq!(parse_int.return_type.as_deref(), Some("i64"));
        assert_eq!(parse_int.params.len(), 2);
        assert_param(parse_int, 0, "text", "string");
        assert_param(parse_int, 1, "radix", "i64");

        let parse_float = stdlib.get("parseFloat").expect("parseFloat must exist");
        assert_eq!(parse_float.return_type.as_deref(), Some("f64"));
        assert_eq!(parse_float.params.len(), 1);
        assert_param(parse_float, 0, "text", "string");

        let includes = stdlib.get("includes").expect("includes must exist");
        assert_eq!(includes.return_type.as_deref(), Some("bool"));
        assert_eq!(includes.params.len(), 2);

        let map = stdlib.get("map").expect("map must exist");
        assert_eq!(map.return_type.as_deref(), Some("array"));
        assert_eq!(map.params.len(), 2);
        assert_param(map, 0, "arr", "array");
        assert_param(map, 1, "fn", "function");

        let reduce = stdlib.get("reduce").expect("reduce must exist");
        assert_eq!(reduce.return_type.as_deref(), Some("any"));
        assert_eq!(reduce.params.len(), 3);
    }
}
