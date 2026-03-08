//! Symbol table for LightLang

use super::types::Type;
use std::collections::HashMap;

/// Symbol kinds
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Constant,
    Function,
    Parameter,
}

/// Symbol information
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub ty: Type,
    pub scope_level: usize,
}

impl Symbol {
    pub fn new(name: String, kind: SymbolKind, ty: Type, scope_level: usize) -> Self {
        Symbol {
            name,
            kind,
            ty,
            scope_level,
        }
    }
}

/// Scope for symbol table
#[derive(Debug, Clone)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<Box<Scope>>,
    pub level: usize,
}

impl Scope {
    pub fn new(level: usize) -> Self {
        Scope {
            symbols: HashMap::new(),
            parent: None,
            level,
        }
    }
    
    pub fn with_parent(parent: Scope) -> Self {
        let level = parent.level + 1;
        Scope {
            symbols: HashMap::new(),
            parent: Some(Box::new(parent)),
            level,
        }
    }
    
    pub fn insert(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.name.clone(), symbol);
    }
    
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.lookup(name))
        })
    }
    
    pub fn lookup_current(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }
}

/// Symbol table with scope management
#[derive(Debug)]
pub struct SymbolTable {
    pub current_scope: Scope,
    pub scope_stack: Vec<usize>, // Track scope levels
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            current_scope: Scope::new(0),
            scope_stack: vec![0],
        }
    }
    
    pub fn enter_scope(&mut self) {
        let new_scope = Scope::with_parent(self.current_scope.clone());
        self.current_scope = new_scope;
        self.scope_stack.push(self.current_scope.level);
    }
    
    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.current_scope.parent.clone() {
            self.current_scope = *parent;
            self.scope_stack.pop();
        }
    }
    
    pub fn define(&mut self, symbol: Symbol) {
        self.current_scope.insert(symbol);
    }
    
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.current_scope.lookup(name)
    }
    
    pub fn lookup_current(&self, name: &str) -> Option<&Symbol> {
        self.current_scope.lookup_current(name)
    }
    
    pub fn current_scope_level(&self) -> usize {
        self.current_scope.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();
        
        let symbol = Symbol::new("x".to_string(), SymbolKind::Variable, Type::Int, 0);
        table.define(symbol);
        
        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_none());
    }

    #[test]
    fn test_nested_scope() {
        let mut table = SymbolTable::new();
        
        // Define in outer scope
        let symbol = Symbol::new("x".to_string(), SymbolKind::Variable, Type::Int, 0);
        table.define(symbol);
        
        // Enter new scope
        table.enter_scope();
        
        // Should still find x
        assert!(table.lookup("x").is_some());
        
        // Define shadowed x
        let symbol2 = Symbol::new("x".to_string(), SymbolKind::Variable, Type::Float, 1);
        table.define(symbol2);
        
        // Should find shadowed x
        assert_eq!(table.lookup("x").unwrap().ty, Type::Float);
        
        // Exit scope
        table.exit_scope();
        
        // Should find original x
        assert_eq!(table.lookup("x").unwrap().ty, Type::Int);
    }

    #[test]
    fn test_lookup_current_scope_only() {
        let mut table = SymbolTable::new();
        table.define(Symbol::new(
            "x".to_string(),
            SymbolKind::Variable,
            Type::Int,
            0,
        ));
        assert!(table.lookup_current("x").is_some());

        table.enter_scope();
        assert!(table.lookup("x").is_some());
        assert!(table.lookup_current("x").is_none());
        assert_eq!(table.current_scope_level(), 1);
    }
}
