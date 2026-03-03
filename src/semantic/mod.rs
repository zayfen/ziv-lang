//! Semantic analyzer for LightLang

pub mod types;
pub mod symbols;
pub mod type_checker;

pub use types::*;
pub use symbols::*;
pub use type_checker::*;

use crate::parser::ast::*;

/// Semantic analyzer that performs type checking and symbol resolution
#[derive(Debug)]
pub struct SemanticAnalyzer {
    pub type_checker: TypeChecker,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            type_checker: TypeChecker::new(),
        }
    }
    
    /// Analyze a program for semantic errors
    pub fn analyze(&mut self, program: &Program) -> Result<(), String> {
        self.type_checker.check_program(program)
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_program() {
        let program = Program::new(vec![]);
        let mut analyzer = SemanticAnalyzer::new();
        assert!(analyzer.analyze(&program).is_ok());
    }

    #[test]
    fn test_analyze_variable() {
        let mut parser = crate::parser::Parser::new("let x = 42;");
        let program = parser.parse().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
        
        // Check that x was defined with correct type
        let symbol = analyzer.type_checker.symbol_table.lookup("x");
        assert!(symbol.is_some());
        assert_eq!(symbol.unwrap().ty, Type::Int);
    }

    #[test]
    fn test_analyze_binary_expr() {
        let mut parser = crate::parser::Parser::new("let x = 1 + 2;");
        let program = parser.parse().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_undefined_variable_error() {
        let mut parser = crate::parser::Parser::new("let y = x;");
        let program = parser.parse().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined variable"));
    }
}
