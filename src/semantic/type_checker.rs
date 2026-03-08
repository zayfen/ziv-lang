//! Type checker for LightLang

use super::symbols::*;
use super::types::*;
use crate::parser::ast::*;
use crate::stdlib::Stdlib;

pub type TypeCheckResult<T> = Result<T, String>;

/// Type checker
#[derive(Debug)]
pub struct TypeChecker {
    pub symbol_table: SymbolTable,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut checker = TypeChecker {
            symbol_table: SymbolTable::new(),
        };
        checker.register_builtin_functions();
        checker
    }

    fn register_builtin_functions(&mut self) {
        let stdlib = Stdlib::new();
        let scope_level = self.symbol_table.current_scope_level();

        for builtin in stdlib.all_functions() {
            let param_types = builtin
                .params
                .iter()
                .map(|param| Type::from(param.ty.as_str()))
                .collect();
            let return_type = builtin
                .return_type
                .as_deref()
                .map(Type::from)
                .unwrap_or(Type::Void);

            let symbol = Symbol::new(
                builtin.name.clone(),
                SymbolKind::Function,
                Type::Function {
                    params: param_types,
                    return_type: Box::new(return_type),
                },
                scope_level,
            );
            self.symbol_table.define(symbol);
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

            Stmt::Assignment { name, value } => {
                self.check_expr(value)?;

                if self
                    .symbol_table
                    .lookup(name)
                    .map(|symbol| symbol.kind == SymbolKind::Constant)
                    .unwrap_or(false)
                {
                    return Err("Cannot assign to constant".to_string());
                }

                Ok(())
            }

            Stmt::FunctionDecl {
                name,
                params,
                return_type,
                body,
            } => {
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
                    self.symbol_table.exit_scope(); }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtins_registered_in_global_scope() {
        let checker = TypeChecker::new();
        assert!(checker.symbol_table.lookup("print").is_some());
        assert!(checker.symbol_table.lookup("println").is_some());
    }

    #[test]
    fn test_assignment_to_constant_errors() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "x".to_string(),
                type_annotation: None,
                init: Some(Expr::Literal(Literal::Number(1))),
                is_const: true,
            })
            .unwrap();

        let err = checker
            .check_stmt(&Stmt::Assignment {
                name: "x".to_string(),
                value: Expr::Literal(Literal::Number(2)),
            })
            .unwrap_err();
        assert!(err.contains("Cannot assign to constant"));
    }

    #[test]
    fn test_typed_decl_and_assignment_to_mutable_variable() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "x".to_string(),
                type_annotation: Some("int".to_string()),
                init: None,
                is_const: false,
            })
            .unwrap();
        checker
            .check_stmt(&Stmt::Assignment {
                name: "x".to_string(),
                value: Expr::Literal(Literal::Number(2)),
            })
            .unwrap();
    }

    #[test]
    fn test_assignment_lookup_branch_for_mutable_symbol() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "m".to_string(),
                type_annotation: None,
                init: Some(Expr::Literal(Literal::Number(1))),
                is_const: false,
            })
            .unwrap();
        assert!(checker.symbol_table.lookup("m").is_some());
        checker
            .check_stmt(&Stmt::Assignment {
                name: "m".to_string(),
                value: Expr::Literal(Literal::Number(2)),
            })
            .unwrap();
    }

    #[test]
    fn test_binary_type_rules() {
        let mut checker = TypeChecker::new();

        let int_expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Number(1))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Number(2))),
        };
        assert_eq!(checker.check_expr(&int_expr).unwrap(), Type::Int);

        let cmp_expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Number(1))),
            op: BinaryOp::Eq,
            right: Box::new(Expr::Literal(Literal::Number(1))),
        };
        assert_eq!(checker.check_expr(&cmp_expr).unwrap(), Type::Bool);

        let and_expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Boolean(true))),
            op: BinaryOp::And,
            right: Box::new(Expr::Literal(Literal::Boolean(false))),
        };
        assert_eq!(checker.check_expr(&and_expr).unwrap(), Type::Bool);

        let or_expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Boolean(true))),
            op: BinaryOp::Or,
            right: Box::new(Expr::Literal(Literal::Boolean(false))),
        };
        assert_eq!(checker.check_expr(&or_expr).unwrap(), Type::Bool);

        let mismatch = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Number(1))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::String("x".to_string()))),
        };
        let err = checker.check_expr(&mismatch).unwrap_err();
        assert!(err.contains("Type mismatch"));
    }

    #[test]
    fn test_any_type_short_circuit_for_arithmetic() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "x".to_string(),
                type_annotation: None,
                init: None,
                is_const: false,
            })
            .unwrap();
        let expr = Expr::Binary {
            left: Box::new(Expr::Identifier("x".to_string())),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Number(1))),
        };
        assert_eq!(checker.check_expr(&expr).unwrap(), Type::Any);
    }

    #[test]
    fn test_literal_float_type() {
        let mut checker = TypeChecker::new();
        let ty = checker
            .check_expr(&Expr::Literal(Literal::Float(1.25)))
            .unwrap();
        assert_eq!(ty, Type::Float);
    }

    #[test]
    fn test_function_decl_and_call_paths() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::FunctionDecl {
                name: "f".to_string(),
                params: vec![Param {
                    name: "a".to_string(),
                    type_annotation: Some("int".to_string()),
                }],
                return_type: Some("int".to_string()),
                body: vec![Stmt::Return(Some(Expr::Identifier("a".to_string())))],
            })
            .unwrap();

        let call = Expr::Call {
            callee: "f".to_string(),
            args: vec![Expr::Literal(Literal::Number(1))],
        };
        assert_eq!(checker.check_expr(&call).unwrap(), Type::Int);

        checker
            .check_stmt(&Stmt::FunctionDecl {
                name: "g".to_string(),
                params: vec![Param {
                    name: "x".to_string(),
                    type_annotation: None,
                }],
                return_type: None,
                body: vec![Stmt::Return(None)],
            })
            .unwrap();
    }

    #[test]
    fn test_call_error_paths() {
        let mut checker = TypeChecker::new();
        let undefined_err = checker
            .check_expr(&Expr::Call {
                callee: "not_found".to_string(),
                args: vec![],
            })
            .unwrap_err();
        assert!(undefined_err.contains("Undefined function"));

        checker
            .check_stmt(&Stmt::VariableDecl {
                name: "x".to_string(),
                type_annotation: None,
                init: Some(Expr::Literal(Literal::Number(1))),
                is_const: false,
            })
            .unwrap();
        let non_func_err = checker
            .check_expr(&Expr::Call {
                callee: "x".to_string(),
                args: vec![],
            })
            .unwrap_err();
        assert!(non_func_err.contains("is not a function"));
    }

    #[test]
    fn test_scope_statements_paths() {
        let mut checker = TypeChecker::new();
        checker
            .check_stmt(&Stmt::If {
                condition: Expr::Literal(Literal::Boolean(true)),
                then_branch: vec![Stmt::VariableDecl {
                    name: "a".to_string(),
                    type_annotation: None,
                    init: Some(Expr::Literal(Literal::Number(1))),
                    is_const: false,
                }],
                else_branch: Some(vec![Stmt::VariableDecl {
                    name: "b".to_string(),
                    type_annotation: None,
                    init: Some(Expr::Literal(Literal::Number(2))),
                    is_const: false,
                }]),
            })
            .unwrap();

        checker
            .check_stmt(&Stmt::While {
                condition: Expr::Literal(Literal::Boolean(true)),
                body: vec![Stmt::Expression(Expr::Literal(Literal::Number(0)))],
            })
            .unwrap();

        checker
            .check_stmt(&Stmt::Block(vec![Stmt::Return(None)]))
            .unwrap();
    }

    #[test]
    fn test_if_else_scope_exits_back_to_parent_scope() {
        let mut checker = TypeChecker::new();
        let level_before = checker.symbol_table.current_scope_level();
        checker
            .check_stmt(&Stmt::If {
                condition: Expr::Literal(Literal::Boolean(true)),
                then_branch: vec![Stmt::Expression(Expr::Literal(Literal::Number(1)))],
                else_branch: Some(vec![Stmt::Expression(Expr::Literal(Literal::Number(2)))]),
            })
            .unwrap();
        assert_eq!(checker.symbol_table.current_scope_level(), level_before);
    }
}
