//! x86-64 assembly code generator

use crate::codegen::CodeGenerator;
use crate::ir::{IRFunction, IRInstruction, IRModule, IRType, IRValue};
use std::collections::HashMap;

pub struct X86_64Generator {
    var_map: HashMap<String, i32>, // variable name -> stack offset
    stack_size: i32,
}

impl X86_64Generator {
    pub fn new() -> Self {
        X86_64Generator {
            var_map: HashMap::new(),
            stack_size: 0,
        }
    }

    fn alloc_stack(&mut self, name: String) -> i32 {
        self.stack_size += 8;
        let offset = self.stack_size;
        self.var_map.insert(name, offset);
        offset
    }

    fn get_offset(&self, name: &str) -> Option<i32> {
        self.var_map.get(name).copied()
    }
}

impl CodeGenerator for X86_64Generator {
    fn generate(&mut self, module: &IRModule) -> Result<String, String> {
        let mut output = String::new();

        // Assembly header
        output.push_str(".section .text\n");
        output.push_str(".globl _start\n\n");

        // Generate code for each function
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }

        // Entry point
        output.push_str("_start:\n");
        output.push_str("    call main\n");
        output.push_str("    movq %rax, %rdi\n");
        output.push_str("    movq $60, %rax\n");
        output.push_str("    syscall\n");

        Ok(output)
    }
}

impl X86_64Generator {
    fn generate_function(&mut self, func: &IRFunction) -> String {
        let mut output = String::new();

        output.push_str(&format!("{}:\n", func.name));
        output.push_str("    pushq %rbp\n");
        output.push_str("    movq %rsp, %rbp\n");

        // Allocate stack space (we'll adjust this later)
        let stack_space = self.stack_size + 16;
        output.push_str(&format!("    subq ${}, %rsp\n\n", stack_space));

        // Generate instructions
        for instr in &func.instructions {
            output.push_str(&self.generate_instruction(instr));
        }

        output.push_str("\n    leave\n");
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
                            format!("    movq ${}, -{}(%rbp)\n", n, offset)
                        }
                        IRValue::Var(name) => {
                            if let Some(src_offset) = self.get_offset(name) {
                                format!(
                                    "    movq -{}(%rbp), %rax\n    movq %rax, -{}(%rbp)\n",
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
                            "    movq -{}(%rbp), %rax\n    movq %rax, -{}(%rbp)\n",
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

                    // Load lhs into rax
                    match lhs {
                        IRValue::Const(n) => {
                            code.push_str(&format!("    movq ${}, %rax\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    movq -{}(%rbp), %rax\n", offset));
                            }
                        }
                    }

                    // Add rhs
                    match rhs {
                        IRValue::Const(n) => {
                            code.push_str(&format!("    addq ${}, %rax\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    addq -{}(%rbp), %rax\n", offset));
                            }
                        }
                    }

                    // Store result
                    code.push_str(&format!("    movq %rax, -{}(%rbp)\n", dest_offset));
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
                        IRValue::Const(n) => {
                            code.push_str(&format!("    movq ${}, %rax\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    movq -{}(%rbp), %rax\n", offset));
                            }
                        }
                    }

                    match rhs {
                        IRValue::Const(n) => {
                            code.push_str(&format!("    subq ${}, %rax\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    subq -{}(%rbp), %rax\n", offset));
                            }
                        }
                    }

                    code.push_str(&format!("    movq %rax, -{}(%rbp)\n", dest_offset));
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
                        IRValue::Const(n) => {
                            code.push_str(&format!("    movq ${}, %rax\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    movq -{}(%rbp), %rax\n", offset));
                            }
                        }
                    }

                    match rhs {
                        IRValue::Const(n) => {
                            code.push_str(&format!("    imulq ${}, %rax\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    imulq -{}(%rbp), %rax\n", offset));
                            }
                        }
                    }

                    code.push_str(&format!("    movq %rax, -{}(%rbp)\n", dest_offset));
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
                        IRValue::Const(n) => {
                            code.push_str(&format!("    movq ${}, %rax\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    movq -{}(%rbp), %rax\n", offset));
                            }
                        }
                    }

                    code.push_str("    cqto\n"); // Sign extend

                    match rhs {
                        IRValue::Const(n) => {
                            code.push_str(&format!("    movq ${}, %rbx\n", n));
                            code.push_str("    idivq %rbx\n");
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    idivq -{}(%rbp)\n", offset));
                            }
                        }
                    }

                    code.push_str(&format!("    movq %rax, -{}(%rbp)\n", dest_offset));
                    code
                } else {
                    String::new()
                }
            }

            IRInstruction::Ret { value, .. } => {
                if let Some(v) = value {
                    match v {
                        IRValue::Const(n) => {
                            format!("    movq ${}, %rax\n", n)
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                format!("    movq -{}(%rbp), %rax\n", offset)
                            } else {
                                "    xorq %rax, %rax\n".to_string()
                            }
                        }
                        _ => "    xorq %rax, %rax\n".to_string(),
                    }
                } else {
                    "    xorq %rax, %rax\n".to_string()
                }
            }

            IRInstruction::Cmp { dest, op, lhs, rhs } => {
                self.alloc_stack(dest.clone());
                if let Some(dest_offset) = self.get_offset(dest) {
                    let mut code = String::new();

                    // Load lhs into rax
                    match lhs {
                        IRValue::Const(n) => {
                            code.push_str(&format!("    movq ${}, %rax\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    movq -{}(%rbp), %rax\n", offset));
                            }
                        }
                    }

                    // Compare with rhs
                    match rhs {
                        IRValue::Const(n) => {
                            code.push_str(&format!("    cmpq ${}, %rax\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&format!("    cmpq -{}(%rbp), %rax\n", offset));
                            }
                        }
                    }

                    // Set result based on comparison operator
                    // Use setCC instruction to set low byte of result register
                    code.push_str("    xorq %rcx, %rcx\n"); // Clear rcx
                    match op {
                        crate::ir::CmpOp::Eq => code.push_str("    sete %cl\n"),
                        crate::ir::CmpOp::Ne => code.push_str("    setne %cl\n"),
                        crate::ir::CmpOp::Lt => code.push_str("    setl %cl\n"),
                        crate::ir::CmpOp::Le => code.push_str("    setle %cl\n"),
                        crate::ir::CmpOp::Gt => code.push_str("    setg %cl\n"),
                        crate::ir::CmpOp::Ge => code.push_str("    setge %cl\n"),
                    }

                    // Store result (0 or 1)
                    code.push_str(&format!("    movq %rcx, -{}(%rbp)\n", dest_offset));
                    code
                } else {
                    String::new()
                }
            }

            IRInstruction::Call { result, function, args } => {
                let mut code = String::new();

                // x86-64 calling convention: args in rdi, rsi, rdx, rcx, r8, r9
                let arg_regs = ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];

                // Push arguments in reverse order for stack args, or use registers for first 6
                // For simplicity, we use registers for up to 6 args
                for (i, arg) in args.iter().enumerate() {
                    if i < 6 {
                        match arg {
                            IRValue::Const(n) => {
                                code.push_str(&format!("    movq ${}, {}\n", n, arg_regs[i]));
                            }
                            IRValue::Var(name) => {
                                if let Some(offset) = self.get_offset(name) {
                                    code.push_str(&format!("    movq -{}(%rbp), {}\n", offset, arg_regs[i]));
                                }
                            }
                        }
                    }
                }

                // Align stack to 16 bytes before call (System V ABI requirement)
                code.push_str("    pushq %rax\n"); // Save rax
                code.push_str("    pushq %rax\n"); // Align to 16 bytes

                // Call the function
                code.push_str(&format!("    call {}\n", function));

                // Restore stack
                code.push_str("    popq %rcx\n"); // Remove alignment
                code.push_str("    popq %rcx\n"); // Restore original rax value (discarded)

                // Store result if needed
                if let Some(dest) = result {
                    self.alloc_stack(dest.clone());
                    if let Some(dest_offset) = self.get_offset(dest) {
                        code.push_str(&format!("    movq %rax, -{}(%rbp)\n", dest_offset));
                    }
                }

                code
            }

            IRInstruction::Label(name) => format!("{}:\n", name),

            IRInstruction::Jump(label) => format!("    jmp {}\n", label),

            IRInstruction::CondBranch {
                condition,
                true_label,
                false_label,
            } => {
                let mut code = String::new();
                match condition {
                    IRValue::Var(name) => {
                        if let Some(offset) = self.get_offset(name) {
                            code.push_str(&format!("    cmpq $0, -{}(%rbp)\n", offset));
                        }
                    }
                    IRValue::Const(n) => {
                        code.push_str(&format!("    cmpq $0, ${}\n", n));
                    }
                }
                code.push_str(&format!("    jne {}\n", true_label));
                code.push_str(&format!("    jmp {}\n", false_label));
                code
            }
        }
    }
}
