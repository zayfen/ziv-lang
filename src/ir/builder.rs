//! IR Builder - converts AST to IR

use crate::ir::{IRFunction, IRInstruction, IRModule, IRType, IRValue};
use crate::parser::ast::*;
use std::collections::HashMap;

pub struct IRBuilder {
    module: IRModule,
    var_counter: usize,
    label_counter: usize,
    variables: HashMap<String, String>,
    last_expr_value: Option<IRValue>,
    // Track if current block has a terminator (return/branch)
    current_block_terminated: bool,
}

impl IRBuilder {
    pub fn new() -> Self {
        IRBuilder {
            module: IRModule::new(),
            var_counter: 0,
            label_counter: 0,
            variables: HashMap::new(),
            last_expr_value: None,
            current_block_terminated: false,
        }
    }

    fn fresh_var(&mut self) -> String {
        let name = format!("t{}", self.var_counter);
        self.var_counter += 1;
        name
    }

    fn fresh_label(&mut self) -> String {
        let name = format!("L{}", self.label_counter);
        self.label_counter += 1;
        name
    }

    fn add_instr(&mut self, func: &mut IRFunction, instr: IRInstruction) {
        // Label always starts a new block, even if previous was terminated
        if let IRInstruction::Label(label_name) = &instr {
            // If previous block wasn't terminated, add a jump to this label
            // This handles fall-through between basic blocks
            if !self.current_block_terminated {
                func.add_instruction(IRInstruction::Jump(label_name.clone()));
            }
            self.current_block_terminated = false;
            func.add_instruction(instr);
            return;
        }

        // Don't add other instructions if current block is already terminated
        if self.current_block_terminated {
            return;
        }

        // Check if this instruction terminates the block
        match &instr {
            IRInstruction::Ret { .. }
            | IRInstruction::Jump(_)
            | IRInstruction::CondBranch { .. } => {
                self.current_block_terminated = true;
            }
            _ => {}
        }

        func.add_instruction(instr);
    }

    pub fn build(mut self, program: &Program) -> IRModule {
        // First pass: collect all function definitions
        for stmt in &program.statements {
            if let Stmt::FunctionDecl {
                name, params, body, ..
            } = stmt
            {
                // Reset state for each function
                self.current_block_terminated = false;
                self.var_counter = 0;
                self.label_counter = 0;
                self.variables.clear();

                let mut func = IRFunction::new(name.clone(), IRType::I64);

                for (i, param) in params.iter().enumerate() {
                    let ptr = format!("arg{}", i);
                    // Add parameter to function signature
                    func.params.push((ptr.clone(), IRType::I64));
                    self.add_instr(&mut func, IRInstruction::Alloc {
                        dest: ptr.clone(),
                        ty: IRType::I64,
                    });
                    self.variables.insert(param.name.clone(), ptr);
                }

                // Build function body
                for body_stmt in body {
                    self.build_stmt(body_stmt, &mut func);
                }

                // Add implicit return if not present
                self.add_instr(&mut func, IRInstruction::Ret {
                    ty: IRType::I64,
                    value: Some(IRValue::Const(0)),
                });

                self.module.add_function(func);
            }
        }

        // Second pass: build main function with non-function statements
        self.current_block_terminated = false;
        self.var_counter = 0;
        self.variables.clear();

        // Use _user_main to avoid conflict with C runtime's main
        let mut main_func = IRFunction::new("_user_main".to_string(), IRType::I64);

        for stmt in &program.statements {
            match stmt {
                Stmt::FunctionDecl { .. } => {} // Skip, already processed
                _ => self.build_stmt(stmt, &mut main_func),
            }
        }

        let ret_value = if let Some(value) = self.last_expr_value.take() {
            Some(value)
        } else {
            Some(IRValue::Const(0))
        };

        self.add_instr(&mut main_func, IRInstruction::Ret {
            ty: IRType::I64,
            value: ret_value,
        });

        self.module.add_function(main_func);
        self.module
    }

    fn build_stmt(&mut self, stmt: &Stmt, func: &mut IRFunction) {
        match stmt {
            Stmt::Expression(expr) => {
                let value = self.build_expr(expr, func);
                self.last_expr_value = Some(value);
            }

            Stmt::VariableDecl { name, init, .. } => {
                let ptr = self.fresh_var();
                self.add_instr(func, IRInstruction::Alloc {
                    dest: ptr.clone(),
                    ty: IRType::I64,
                });

                if let Some(init_expr) = init {
                    let value = self.build_expr(init_expr, func);
                    self.add_instr(func, IRInstruction::Store {
                        dest: ptr.clone(),
                        ty: IRType::I64,
                        value,
                    });
                    self.last_expr_value = Some(IRValue::Var(ptr.clone()));
                }

                self.variables.insert(name.clone(), ptr);
            }

            Stmt::Assignment { name, value } => {
                if let Some(ptr) = self.variables.get(name).cloned() {
                    let val = self.build_expr(value, func);
                    self.add_instr(func, IRInstruction::Store {
                        dest: ptr,
                        ty: IRType::I64,
                        value: val,
                    });
                }
            }

            Stmt::FunctionDecl { .. } => {}

            Stmt::Return(expr) => {
                let value = if let Some(e) = expr {
                    Some(self.build_expr(e, func))
                } else {
                    None
                };
                self.add_instr(func, IRInstruction::Ret {
                    ty: IRType::I64,
                    value,
                });
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.build_expr(condition, func);
                let then_label = self.fresh_label();
                let else_label = self.fresh_label();
                let end_label = self.fresh_label();

                self.add_instr(func, IRInstruction::CondBranch {
                    condition: cond_val,
                    true_label: then_label.clone(),
                    false_label: else_label.clone(),
                });

                // Then branch
                self.add_instr(func, IRInstruction::Label(then_label));
                let then_terminated = self.current_block_terminated;
                for stmt in then_branch {
                    self.build_stmt(stmt, func);
                }
                // Only add jump if block wasn't terminated by return
                if !self.current_block_terminated {
                    self.add_instr(func, IRInstruction::Jump(end_label.clone()));
                }

                // Else branch
                self.add_instr(func, IRInstruction::Label(else_label));
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.build_stmt(stmt, func);
                    }
                }
                // Only add jump if block wasn't terminated by return
                if !self.current_block_terminated {
                    self.add_instr(func, IRInstruction::Jump(end_label.clone()));
                }

                // End label
                self.add_instr(func, IRInstruction::Label(end_label));
            }

            Stmt::While { condition, body } => {
                let start_label = self.fresh_label();
                let body_label = self.fresh_label();
                let end_label = self.fresh_label();

                self.add_instr(func, IRInstruction::Label(start_label.clone()));
                let cond_val = self.build_expr(condition, func);
                self.add_instr(func, IRInstruction::CondBranch {
                    condition: cond_val,
                    true_label: body_label.clone(),
                    false_label: end_label.clone(),
                });

                self.add_instr(func, IRInstruction::Label(body_label));
                for stmt in body {
                    self.build_stmt(stmt, func);
                }
                // Jump back to start if not terminated
                if !self.current_block_terminated {
                    self.add_instr(func, IRInstruction::Jump(start_label));
                }

                self.add_instr(func, IRInstruction::Label(end_label));
            }

            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.build_stmt(stmt, func);
                }
            }
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
                    self.add_instr(func, IRInstruction::Load {
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
                    BinaryOp::Eq => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Eq,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Ne => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Ne,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Lt => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Lt,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Le => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Le,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Gt => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Gt,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Ge => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Ge,
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

                self.add_instr(func, instr);
                IRValue::Var(dest)
            }

            Expr::Call { callee, args } => {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.build_expr(arg, func));
                }

                let dest = self.fresh_var();
                self.add_instr(func, IRInstruction::Call {
                    result: Some(dest.clone()),
                    function: callee.clone(),
                    args: arg_values,
                });
                IRValue::Var(dest)
            }
        }
    }
}
