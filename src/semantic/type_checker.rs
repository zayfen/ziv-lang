//! Type checker for LightLang

use crate::parser::ast::*;
use super::symbols::*;
use super::types::*;

pub type TypeCheckResult<T> = Result<T, String>;

/// Type checker
#[derive(Debug)]
pub struct TypeChecker {
    pub symbol_table: SymbolTable,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            symbol_table: SymbolTable::new(),
        }
    }
    
    pub fn check_program(&mut self, program: &Program) -> TypeCheckResult<()> {
        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }
    
    pub fn check_stmt(&mut self, stmt: &Stmt) -> TypeCheckResult<()> {
        match stmt {
            Stmt::Expression(expr) => {
                self.check_expr(expr)?;
                Ok(())
            },
            
            Stmt::VariableDecl { name, init, is_const } => {
                let ty = if let Some(init_expr) = init {
                    self.check_expr(init_expr)?
                } else {
                    Type::Any
                };
                
                let kind = if *is_const {
                    SymbolKind::Constant
                } else {
                    SymbolKind::Variable
                };
                
                let scope_level = self.symbol_table.current_scope_level();
                let symbol = Symbol::new(name.clone(), kind, ty, scope_level);
                self.symbol_table.define(symbol);
                
                Ok(())
            },
        }
    }
    
    pub fn check_expr(&mut self, expr: &Expr) -> TypeCheckResult<Type> {
        match expr {
            Expr::Literal(lit) => {
                let ty = match lit {
                    Literal::Number(_) => Type::Int,
                    _ => Type::Any,
                };
                Ok(ty)
            },
            
            Expr::Identifier(name) => {
                self.symbol_table.lookup(name)
                    .map(|s| s.ty.clone())
                    .ok_or_else(|| format!("Undefined variable: {}", name))
            },
            
            Expr::Binary { left, op, right } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;
                
                let result_type = match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                        if left_type == Type::Any || right_type == Type::Any {
                            Type::Any
                        } else if left_type == Type::Int && right_type == Type::Int {
                            Type::Int
                        } else {
                            return Err(format!("Type mismatch: {} and {}", left_type, right_type));
                        }
                    },
                };
                
                Ok(result_type)
            },
        }
    }
}
