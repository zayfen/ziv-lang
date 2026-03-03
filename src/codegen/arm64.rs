use crate::codegen::CodeGenerator;
use crate::ir::{IRFunction, IRInstruction, IRModule, IRType, IRValue};
use std::collections::HashMap;

pub struct ARM64Generator {
    var_map: HashMap<String, i32>,
    stack_size: i32,
}

impl ARM64Generator {
    pub fn new() -> Self {
        ARM64Generator {
            var_map: HashMap::new(),
            stack_size: 0,
        }
    }

    fn alloc_stack(&mut self, name: String) -> i32 {
        self.stack_size += 16;
        let offset = self.stack_size;
        self.var_map.insert(name, offset);
        offset
    }

    fn get_offset(&self, name: &str) -> Option<i32> {
        self.var_map.get(name).copied()
    }
}

impl CodeGenerator for ARM64Generator {
    fn generate(&mut self, module: &IRModule) -> Result<String, String> {
        let mut output = String::new();

        output.push_str(".section .text\n");
        output.push_str(".globl _start\n\n");

        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }

        output.push_str("_start:\n");
        output.push_str("    bl main\n");
        output.push_str("    mov x8, #93\n");
        output.push_str("    svc #0\n");

        Ok(output)
    }
}

impl ARM64Generator {
    fn generate_function(&mut self, func: &IRFunction) -> String {
        let mut output = String::new();

        output.push_str(&format!("{}:\n", func.name));
        output.push_str("    stp x29, x30, [sp, #-16]!\n");
        output.push_str("    mov x29, sp\n");

        let stack_space = self.stack_size + 32;
        output.push_str(&format!("    sub sp, sp, #{}\n\n", stack_space));

        for instr in &func.instructions {
            output.push_str(&self.generate_instruction(instr));
        }

        output.push_str("\n    mov sp, x29\n");
        output.push_str("    ldp x29, x30, [sp], #16\n");
        output.push_str("    ret\n\n");

        output
    }

    fn generate_instruction(&mut self, instr: &IRInstruction) -> String {
        match instr {
            IRInstruction::Alloc { dest, .. } => {
                self.alloc_stack(dest.clone());
                String::new()
            }

            IRInstruction::Store { dest, value, .. } => {
                if let Some(offset) = self.get_offset(dest) {
                    match value {
                        IRValue::Const(n) => {
                            format!("    mov x0, #{}\n    str x0, [x29, #-{}]\n", n, offset)
                        }
                        IRValue::Var(name) => {
                            if let Some(src_offset) = self.get_offset(name) {
                                format!(
                                    "    ldr x0, [x29, #-{}]\n    str x0, [x29, #-{}]\n",
                                    src_offset, offset
                                )
                            } else {
                                String::new()
                            }
                        }
                    }
                } else {
                    String::new()
                }
            }

            IRInstruction::Load { dest, ptr, .. } => {
                if let Some(offset) = self.get_offset(ptr) {
                    self.alloc_stack(dest.clone());
                    if let Some(dest_offset) = self.get_offset(dest) {
                        format!(
                            "    ldr x0, [x29, #-{}]\n    str x0, [x29, #-{}]\n",
                            offset, dest_offset
                        )
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            }

            IRInstruction::Add { dest, lhs, rhs, .. } => {
                self.alloc_stack(dest.clone());
                if let Some(dest_offset) = self.get_offset(dest) {
                    let mut code = String::new();

                    match lhs {
                        IRValue::Const(n) => {
                            code.push_str(&format!("    mov x0, #{}\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    ldr x0, [x29, #-{}]\n", offset));
                            }
                        }
                    }

                    match rhs {
                        IRValue::Const(n) => {
                            code.push_str(&format!("    add x0, x0, #{}\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!(
                                    "    ldr x1, [x29, #-{}]\n    add x0, x0, x1\n",
                                    offset
                                ));
                            }
                        }
                    }

                    code.push_str(&format!("    str x0, [x29, #-{}]\n", dest_offset));
                    code
                } else {
                    String::new()
                }
            }

            IRInstruction::Sub { dest, lhs, rhs, .. } => {
                self.alloc_stack(dest.clone());
                if let Some(dest_offset) = self.get_offset(dest) {
                    let mut code = String::new();

                    match lhs {
                        IRValue::Const(n) => code.push_str(&format!("    mov x0, #{}\n", n)),
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    ldr x0, [x29, #-{}]\n", offset));
                            }
                        }
                    }

                    match rhs {
                        IRValue::Const(n) => code.push_str(&format!("    sub x0, x0, #{}\n", n)),
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!(
                                    "    ldr x1, [x29, #-{}]\n    sub x0, x0, x1\n",
                                    offset
                                ));
                            }
                        }
                    }

                    code.push_str(&format!("    str x0, [x29, #-{}]\n", dest_offset));
                    code
                } else {
                    String::new()
                }
            }

            IRInstruction::Mul { dest, lhs, rhs, .. } => {
                self.alloc_stack(dest.clone());
                if let Some(dest_offset) = self.get_offset(dest) {
                    let mut code = String::new();

                    match lhs {
                        IRValue::Const(n) => code.push_str(&format!("    mov x0, #{}\n", n)),
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    ldr x0, [x29, #-{}]\n", offset));
                            }
                        }
                    }

                    match rhs {
                        IRValue::Const(n) => code.push_str(&format!("    mul x0, x0, #{}\n", n)),
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!(
                                    "    ldr x1, [x29, #-{}]\n    mul x0, x0, x1\n",
                                    offset
                                ));
                            }
                        }
                    }

                    code.push_str(&format!("    str x0, [x29, #-{}]\n", dest_offset));
                    code
                } else {
                    String::new()
                }
            }

            IRInstruction::Div { dest, lhs, rhs, .. } => {
                self.alloc_stack(dest.clone());
                if let Some(dest_offset) = self.get_offset(dest) {
                    let mut code = String::new();

                    match lhs {
                        IRValue::Const(n) => code.push_str(&format!("    mov x0, #{}\n", n)),
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    ldr x0, [x29, #-{}]\n", offset));
                            }
                        }
                    }

                    match rhs {
                        IRValue::Const(n) => {
                            code.push_str(&format!("    mov x1, #{}\n    sdiv x0, x0, x1\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!(
                                    "    ldr x1, [x29, #-{}]\n    sdiv x0, x0, x1\n",
                                    offset
                                ));
                            }
                        }
                    }

                    code.push_str(&format!("    str x0, [x29, #-{}]\n", dest_offset));
                    code
                } else {
                    String::new()
                }
            }

            IRInstruction::Cmp { dest, op, lhs, rhs } => {
                self.alloc_stack(dest.clone());
                if let Some(dest_offset) = self.get_offset(dest) {
                    let mut code = String::new();

                    match lhs {
                        IRValue::Const(n) => code.push_str(&format!("    mov x0, #{}\n", n)),
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    ldr x0, [x29, #-{}]\n", offset));
                            }
                        }
                    }

                    match rhs {
                        IRValue::Const(n) => code.push_str(&format!("    cmp x0, #{}\n", n)),
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!(
                                    "    ldr x1, [x29, #-{}]\n    cmp x0, x1\n",
                                    offset
                                ));
                            }
                        }
                    }

                    let cond = match op {
                        crate::ir::CmpOp::Eq => "eq",
                        crate::ir::CmpOp::Ne => "ne",
                        crate::ir::CmpOp::Lt => "lt",
                        crate::ir::CmpOp::Le => "le",
                        crate::ir::CmpOp::Gt => "gt",
                        crate::ir::CmpOp::Ge => "ge",
                    };

                    code.push_str(&format!("    cset x0, {}\n", cond));
                    code.push_str(&format!("    str x0, [x29, #-{}]\n", dest_offset));
                    code
                } else {
                    String::new()
                }
            }

            IRInstruction::Call {
                result,
                function,
                args,
            } => {
                let mut code = String::new();

                let arg_regs = ["x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7"];

                for (i, arg) in args.iter().enumerate() {
                    if i < arg_regs.len() {
                        match arg {
                            IRValue::Const(n) => {
                                code.push_str(&format!("    mov {}, #{}\n", arg_regs[i], n));
                            }
                            IRValue::Var(name) => {
                                if let Some(offset) = self.get_offset(name) {
                                    code.push_str(&format!(
                                        "    ldr {}, [x29, #-{}]\n",
                                        arg_regs[i], offset
                                    ));
                                }
                            }
                        }
                    }
                }

                code.push_str(&format!("    bl {}\n", function));

                if let Some(res) = result {
                    self.alloc_stack(res.clone());
                    if let Some(res_offset) = self.get_offset(res) {
                        code.push_str(&format!("    str x0, [x29, #-{}]\n", res_offset));
                    }
                }

                code
            }

            IRInstruction::Label(name) => format!("{}:\n", name),

            IRInstruction::Jump(label) => format!("    b {}\n", label),

            IRInstruction::CondBranch {
                condition,
                true_label,
                false_label,
            } => {
                let mut code = String::new();
                match condition {
                    IRValue::Var(name) => {
                        if let Some(offset) = self.get_offset(name) {
                            code.push_str(&format!("    ldr x0, [x29, #-{}]\n", offset));
                            code.push_str("    cmp x0, #0\n");
                        }
                    }
                    IRValue::Const(n) => {
                        code.push_str(&format!("    cmp x0, #{}\n", n));
                    }
                }
                code.push_str(&format!("    b.ne {}\n", true_label));
                code.push_str(&format!("    b {}\n", false_label));
                code
            }

            IRInstruction::Ret { value, .. } => {
                if let Some(v) = value {
                    match v {
                        IRValue::Const(n) => format!("    mov x0, #{}\n", n),
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                format!("    ldr x0, [x29, #-{}]\n", offset)
                            } else {
                                "    mov x0, #0\n".to_string()
                            }
                        }
                    }
                } else {
                    "    mov x0, #0\n".to_string()
                }
            }
        }
    }
}
