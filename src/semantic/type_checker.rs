//! Type checker for LightLang

use super::symbols::*;
use super::types::*;
use crate::parser::ast::*;

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
            }

            Stmt::VariableDecl {
                name,
                type_annotation,
                init,
                is_const,
            } => {
                let ty = if let Some(init_expr) = init {
                    self.check_expr(init_expr)?
                } else if let Some(type_name) = type_annotation {
                    Type::from(type_name.as_str())
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
            }

            Stmt::FunctionDecl {
                name,
                params,
                return_type,
                body,
            } => {
                self.symbol_table.enter_scope();

                for param in params {
                    let param_type = if let Some(type_name) = &param.type_annotation {
                        Type::from(type_name.as_str())
                    } else {
                        Type::Any
                    };

                    let symbol = Symbol::new(
                        param.name.clone(),
                        SymbolKind::Parameter,
                        param_type,
                        self.symbol_table.current_scope_level(),
                    );
                    self.symbol_table.define(symbol);
                }

                for body_stmt in body {
                    self.check_stmt(body_stmt)?;
                }

                self.symbol_table.exit_scope();

                let func_type = Type::Function {
                    params: params
                        .iter()
                        .map(|p| {
                            if let Some(type_name) = &p.type_annotation {
                                Type::from(type_name.as_str())
                            } else {
                                Type::Any
                            }
                        })
                        .collect(),
                    return_type: Box::new(if let Some(ret_type) = return_type {
                        Type::from(ret_type.as_str())
                    } else {
                        Type::Any
                    }),
                };

                let symbol = Symbol::new(
                    name.clone(),
                    SymbolKind::Function,
                    func_type,
                    self.symbol_table.current_scope_level(),
                );
                self.symbol_table.define(symbol);

                Ok(())
            }

            Stmt::Return(value) => {
                if let Some(expr) = value {
                    self.check_expr(expr)?;
                }
                Ok(())
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.check_expr(condition)?;

                self.symbol_table.enter_scope();
                for stmt in then_branch {
                    self.check_stmt(stmt)?;
                }
                self.symbol_table.exit_scope();

                if let Some(else_stmts) = else_branch {
                    self.symbol_table.enter_scope();
                    for stmt in else_stmts {
                        self.check_stmt(stmt)?;
                    }
                    self.symbol_table.exit_scope();
                }

                Ok(())
            }

            Stmt::While { condition, body } => {
                self.check_expr(condition)?;

                self.symbol_table.enter_scope();
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                self.symbol_table.exit_scope();

                Ok(())
            }

            Stmt::Block(stmts) => {
                self.symbol_table.enter_scope();
                for stmt in stmts {
                    self.check_stmt(stmt)?;
                }
                self.symbol_table.exit_scope();
                Ok(())
            }
        }
    }

    pub fn check_expr(&mut self, expr: &Expr) -> TypeCheckResult<Type> {
        match expr {
            Expr::Literal(lit) => {
                let ty = match lit {
                    Literal::Number(_) => Type::Int,
                    Literal::Float(_) => Type::Float,
                    Literal::String(_) => Type::String,
                    Literal::Boolean(_) => Type::Bool,
                };
                Ok(ty)
            }

            Expr::Identifier(name) => self
                .symbol_table
                .lookup(name)
                .map(|s| s.ty.clone())
                .ok_or_else(|| format!("Undefined variable: {}", name)),

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
                    }

                    BinaryOp::Eq
                    | BinaryOp::Ne
                    | BinaryOp::Lt
                    | BinaryOp::Le
                    | BinaryOp::Gt
                    | BinaryOp::Ge => Type::Bool,

                    BinaryOp::And | BinaryOp::Or => Type::Bool,
                };

                Ok(result_type)
            }

            Expr::Call { callee, args } => {
                // Check arguments
                for arg in args {
                    self.check_expr(arg)?;
                }

                // Look up function
                let func_type = self
                    .symbol_table
                    .lookup(callee)
                    .map(|s| s.ty.clone())
                    .ok_or_else(|| format!("Undefined function: {}", callee))?;

                // Return the function's return type
                match func_type {
                    Type::Function { return_type, .. } => Ok(*return_type),
                    _ => Err(format!("{} is not a function", callee)),
                }
            }
        }
    }
}
