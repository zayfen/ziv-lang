//! IR Builder - converts AST to IR

use crate::ir::{IRFunction, IRInstruction, IRModule, IRType, IRValue};
use crate::parser::ast::*;
use std::collections::HashMap;

pub struct IRBuilder {
    module: IRModule,
    var_counter: usize,
    variables: HashMap<String, String>,
}

impl IRBuilder {
    pub fn new() -> Self {
        IRBuilder {
            module: IRModule::new(),
            var_counter: 0,
            variables: HashMap::new(),
        }
    }

    fn fresh_var(&mut self) -> String {
        let name = format!("t{}", self.var_counter);
        self.var_counter += 1;
        name
    }

    pub fn build(mut self, program: &Program) -> IRModule {
        let mut main_func = IRFunction::new("main".to_string(), IRType::I64);

        for stmt in &program.statements {
            self.build_stmt(stmt, &mut main_func);
        }

        main_func.add_instruction(IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Const(0)),
        });

        self.module.add_function(main_func);
        self.module
    }

    fn build_stmt(&mut self, stmt: &Stmt, func: &mut IRFunction) {
        match stmt {
            Stmt::Expression(expr) => {
                self.build_expr(expr, func);
            }

            Stmt::VariableDecl { name, init, .. } => {
                let ptr = self.fresh_var();
                func.add_instruction(IRInstruction::Alloc {
                    dest: ptr.clone(),
                    ty: IRType::I64,
                });

                if let Some(init_expr) = init {
                    let value = self.build_expr(init_expr, func);
                    func.add_instruction(IRInstruction::Store {
                        dest: ptr.clone(),
                        ty: IRType::I64,
                        value,
                    });
                }

                self.variables.insert(name.clone(), ptr);
            }

            Stmt::FunctionDecl { .. } => {}
            Stmt::Return(_) => {}
            Stmt::If { .. } => {}
            Stmt::While { .. } => {}
            Stmt::Block(_) => {}
        }
    }

    fn build_expr(&mut self, expr: &Expr, func: &mut IRFunction) -> IRValue {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Number(n) => IRValue::Const(*n),
                _ => IRValue::Const(0),
            },

            Expr::Identifier(name) => {
                if let Some(ptr) = self.variables.get(name).cloned() {
                    let dest = self.fresh_var();
                    func.add_instruction(IRInstruction::Load {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        ptr: ptr,
                    });
                    IRValue::Var(dest)
                } else {
                    IRValue::Const(0)
                }
            }

            Expr::Binary { left, op, right } => {
                let lhs = self.build_expr(left, func);
                let rhs = self.build_expr(right, func);
                let dest = self.fresh_var();

                let instr = match op {
                    BinaryOp::Add => IRInstruction::Add {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Sub => IRInstruction::Sub {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Mul => IRInstruction::Mul {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Div => IRInstruction::Div {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs,
                        rhs,
                    },
                    _ => IRInstruction::Add {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs: IRValue::Const(0),
                        rhs: IRValue::Const(0),
                    },
                };

                func.add_instruction(instr);
                IRValue::Var(dest)
            }

            Expr::Call { callee, args } => {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.build_expr(arg, func));
                }

                let dest = self.fresh_var();
                func.add_instruction(IRInstruction::Call {
                    result: Some(dest.clone()),
                    function: callee.clone(),
                    args: arg_values,
                });
                IRValue::Var(dest)
            }
        }
    }
}
