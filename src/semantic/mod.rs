//! Semantic analyzer for LightLang

/// Semantic analyzer
#[derive(Debug, Default)]
pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer
    }
    
    /// Analyze a program
    pub fn analyze(&self, _program: &crate::parser::ast::Program) -> Result<(), String> {
        Ok(())
    }
}
