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

    fn symbol_name(name: &str) -> String {
        #[cfg(target_os = "macos")]
        {
            format!("_{}", name)
        }
        #[cfg(not(target_os = "macos"))]
        {
            name.to_string()
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
        output.push_str(".text\n");
        output.push_str(".globl _start\n\n");

        // Generate code for each function
        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }

        // Entry point
        output.push_str("_start:\n");
        output.push_str(&format!("    call {}\n", Self::symbol_name("main")));
        output.push_str("    movq %rax, %rdi\n");
        output.push_str("    movq $60, %rax\n");
        output.push_str("    syscall\n");

        Ok(output)
    }
}

impl X86_64Generator {
    fn generate_function(&mut self, func: &IRFunction) -> String {
        let mut output = String::new();

        output.push_str(&format!("{}:\n", Self::symbol_name(&func.name)));
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
                        IRValue::Str(_) => format!("    movq $0, -{}(%rbp)\n", offset),
                    }
                } else {
                    String::new()
                }
            }

            IRInstruction::Load { dest, ptr, .. } => {
                if let Some(offset) = self.get_offset(ptr) {
                    self.alloc_stack(dest.clone());
                    let dest_offset = self
                        .get_offset(dest)
                        .expect("allocated destination must have a stack slot");
                    format!(
                        "    movq -{}(%rbp), %rax\n    movq %rax, -{}(%rbp)\n",
                        offset, dest_offset
                    )
                } else {
                    String::new()
                }
            }

            IRInstruction::Add { dest, lhs, rhs, .. } => {
                self.alloc_stack(dest.clone());
                let dest_offset = self.get_offset(dest).expect("allocated dest must exist");
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
                    IRValue::Str(_) => code.push_str("    movq $0, %rax\n"),
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
                    IRValue::Str(_) => code.push_str("    addq $0, %rax\n"),
                }

                // Store result
                code.push_str(&format!("    movq %rax, -{}(%rbp)\n", dest_offset));
                code
            }

            IRInstruction::Sub { dest, lhs, rhs, .. } => {
                self.alloc_stack(dest.clone());
                let dest_offset = self.get_offset(dest).expect("allocated dest must exist");
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
                    IRValue::Str(_) => code.push_str("    movq $0, %rax\n"),
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
                    IRValue::Str(_) => code.push_str("    subq $0, %rax\n"),
                }

                code.push_str(&format!("    movq %rax, -{}(%rbp)\n", dest_offset));
                code
            }

            IRInstruction::Mul { dest, lhs, rhs, .. } => {
                self.alloc_stack(dest.clone());
                let dest_offset = self.get_offset(dest).expect("allocated dest must exist");
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
                    IRValue::Str(_) => code.push_str("    movq $0, %rax\n"),
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
                    IRValue::Str(_) => code.push_str("    imulq $0, %rax\n"),
                }

                code.push_str(&format!("    movq %rax, -{}(%rbp)\n", dest_offset));
                code
            }

            IRInstruction::Div { dest, lhs, rhs, .. } => {
                self.alloc_stack(dest.clone());
                let dest_offset = self.get_offset(dest).expect("allocated dest must exist");
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
                    IRValue::Str(_) => code.push_str("    movq $0, %rax\n"),
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
                    IRValue::Str(_) => {
                        code.push_str("    movq $1, %rbx\n");
                        code.push_str("    idivq %rbx\n");
                    }
                }

                code.push_str(&format!("    movq %rax, -{}(%rbp)\n", dest_offset));
                code
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
                        IRValue::Str(_) => "    xorq %rax, %rax\n".to_string(),
                    }
                } else {
                    "    xorq %rax, %rax\n".to_string()
                }
            }

            IRInstruction::Cmp { dest, op, lhs, rhs } => {
                self.alloc_stack(dest.clone());
                let dest_offset = self.get_offset(dest).expect("allocated dest must exist");
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
                    IRValue::Str(_) => code.push_str("    movq $0, %rax\n"),
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
                    IRValue::Str(_) => code.push_str("    cmpq $0, %rax\n"),
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
                            IRValue::Str(_) => {
                                code.push_str(&format!("    movq $0, {}\n", arg_regs[i]));
                            }
                        }
                    }
                }

                // Align stack to 16 bytes before call (System V ABI requirement)
                code.push_str("    pushq %rax\n"); // Save rax
                code.push_str("    pushq %rax\n"); // Align to 16 bytes

                // Call the function
                code.push_str(&format!("    call {}\n", Self::symbol_name(function)));

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
                    IRValue::Str(_) => {
                        code.push_str("    cmpq $0, $1\n");
                    }
                }
                code.push_str(&format!("    jne {}\n", true_label));
                code.push_str(&format!("    jmp {}\n", false_label));
                code
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{CmpOp, IRInstruction, IRType};

    fn alloc(gen: &mut X86_64Generator, name: &str) {
        let _ = gen.generate_instruction(&IRInstruction::Alloc {
            dest: name.to_string(),
            ty: IRType::I64,
        });
    }

    #[test]
    fn test_generate_instruction_variants() {
        let mut gen = X86_64Generator::new();
        for name in ["a", "b", "cond"] {
            alloc(&mut gen, name);
        }

        let store_const = gen.generate_instruction(&IRInstruction::Store {
            dest: "a".to_string(),
            ty: IRType::I64,
            value: IRValue::Const(1),
        });
        assert!(store_const.contains("movq $1, -8(%rbp)"));

        let store_var = gen.generate_instruction(&IRInstruction::Store {
            dest: "b".to_string(),
            ty: IRType::I64,
            value: IRValue::Var("a".to_string()),
        });
        assert!(store_var.contains("movq -8(%rbp), %rax"));
        assert_eq!(
            gen.generate_instruction(&IRInstruction::Store {
                dest: "b".to_string(),
                ty: IRType::I64,
                value: IRValue::Var("missing".to_string()),
            }),
            ""
        );

        assert_eq!(
            gen.generate_instruction(&IRInstruction::Store {
                dest: "missing".to_string(),
                ty: IRType::I64,
                value: IRValue::Const(1),
            }),
            ""
        );

        let load_ok = gen.generate_instruction(&IRInstruction::Load {
            dest: "c".to_string(),
            ty: IRType::I64,
            ptr: "a".to_string(),
        });
        assert!(load_ok.contains("movq -8(%rbp), %rax"));
        let load_missing = gen.generate_instruction(&IRInstruction::Load {
            dest: "d".to_string(),
            ty: IRType::I64,
            ptr: "missing".to_string(),
        });
        assert_eq!(load_missing, "");

        let add = gen.generate_instruction(&IRInstruction::Add {
            dest: "e".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Const(1),
            rhs: IRValue::Var("a".to_string()),
        });
        assert!(add.contains("addq"));
        let add_var_const = gen.generate_instruction(&IRInstruction::Add {
            dest: "e2".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("a".to_string()),
            rhs: IRValue::Const(3),
        });
        assert!(add_var_const.contains("movq -8(%rbp), %rax"));
        assert!(add_var_const.contains("addq $3, %rax"));

        let sub = gen.generate_instruction(&IRInstruction::Sub {
            dest: "f".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("a".to_string()),
            rhs: IRValue::Const(1),
        });
        assert!(sub.contains("subq"));
        let sub_const_var = gen.generate_instruction(&IRInstruction::Sub {
            dest: "f2".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Const(7),
            rhs: IRValue::Var("a".to_string()),
        });
        assert!(sub_const_var.contains("movq $7, %rax"));
        assert!(sub_const_var.contains("subq -8(%rbp), %rax"));

        let mul = gen.generate_instruction(&IRInstruction::Mul {
            dest: "g".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Const(2),
            rhs: IRValue::Var("a".to_string()),
        });
        assert!(mul.contains("imulq"));
        let mul_var_const = gen.generate_instruction(&IRInstruction::Mul {
            dest: "g2".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("a".to_string()),
            rhs: IRValue::Const(2),
        });
        assert!(mul_var_const.contains("movq -8(%rbp), %rax"));
        assert!(mul_var_const.contains("imulq $2, %rax"));

        let div = gen.generate_instruction(&IRInstruction::Div {
            dest: "h".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("a".to_string()),
            rhs: IRValue::Const(2),
        });
        assert!(div.contains("idivq"));
        let div_const_var = gen.generate_instruction(&IRInstruction::Div {
            dest: "h2".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Const(8),
            rhs: IRValue::Var("a".to_string()),
        });
        assert!(div_const_var.contains("movq $8, %rax"));
        assert!(div_const_var.contains("idivq -8(%rbp)"));

        let ret_const = gen.generate_instruction(&IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Const(9)),
        });
        assert!(ret_const.contains("movq $9, %rax"));
        let ret_var_missing = gen.generate_instruction(&IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Var("missing".to_string())),
        });
        assert!(ret_var_missing.contains("xorq %rax, %rax"));
        let ret_none = gen.generate_instruction(&IRInstruction::Ret {
            ty: IRType::I64,
            value: None,
        });
        assert!(ret_none.contains("xorq %rax, %rax"));

        for (op, expected) in [
            (CmpOp::Eq, "sete"),
            (CmpOp::Ne, "setne"),
            (CmpOp::Lt, "setl"),
            (CmpOp::Le, "setle"),
            (CmpOp::Gt, "setg"),
            (CmpOp::Ge, "setge"),
        ] {
            let cmp = gen.generate_instruction(&IRInstruction::Cmp {
                dest: "cmp".to_string(),
                op,
                lhs: IRValue::Const(1),
                rhs: IRValue::Var("a".to_string()),
            });
            assert!(cmp.contains(expected));
        }
        let cmp_var_const = gen.generate_instruction(&IRInstruction::Cmp {
            dest: "cmp2".to_string(),
            op: CmpOp::Eq,
            lhs: IRValue::Var("a".to_string()),
            rhs: IRValue::Const(1),
        });
        assert!(cmp_var_const.contains("movq -8(%rbp), %rax"));
        assert!(cmp_var_const.contains("cmpq $1, %rax"));

        let call = gen.generate_instruction(&IRInstruction::Call {
            result: Some("res".to_string()),
            function: "foo".to_string(),
            args: vec![
                IRValue::Const(1),
                IRValue::Var("a".to_string()),
                IRValue::Const(3),
            ],
        });
        #[cfg(target_os = "macos")]
        assert!(call.contains("call _foo"));
        #[cfg(not(target_os = "macos"))]
        assert!(call.contains("call foo"));
        assert!(call.contains("movq %rax"));
        let call_no_result = gen.generate_instruction(&IRInstruction::Call {
            result: None,
            function: "bar".to_string(),
            args: vec![IRValue::Const(1)],
        });
        #[cfg(target_os = "macos")]
        assert!(call_no_result.contains("call _bar"));
        #[cfg(not(target_os = "macos"))]
        assert!(call_no_result.contains("call bar"));
        let many_args_call = gen.generate_instruction(&IRInstruction::Call {
            result: None,
            function: "many".to_string(),
            args: vec![
                IRValue::Const(1),
                IRValue::Const(2),
                IRValue::Const(3),
                IRValue::Const(4),
                IRValue::Const(5),
                IRValue::Const(6),
                IRValue::Const(7),
            ],
        });
        assert!(many_args_call.contains("movq $6, %r9"));
        #[cfg(target_os = "macos")]
        assert!(many_args_call.contains("call _many"));
        #[cfg(not(target_os = "macos"))]
        assert!(many_args_call.contains("call many"));

        assert_eq!(
            gen.generate_instruction(&IRInstruction::Label("L1".to_string())),
            "L1:\n"
        );
        assert_eq!(
            gen.generate_instruction(&IRInstruction::Jump("L2".to_string())),
            "    jmp L2\n"
        );

        let br_var = gen.generate_instruction(&IRInstruction::CondBranch {
            condition: IRValue::Var("cond".to_string()),
            true_label: "T".to_string(),
            false_label: "F".to_string(),
        });
        assert!(br_var.contains("jne T"));
        let br_const = gen.generate_instruction(&IRInstruction::CondBranch {
            condition: IRValue::Const(1),
            true_label: "T2".to_string(),
            false_label: "F2".to_string(),
        });
        assert!(br_const.contains("cmpq $0, $1"));
    }

    #[test]
    fn test_generate_module_output() {
        let mut func = IRFunction::new("main".to_string(), IRType::I64);
        func.instructions.push(IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Const(0)),
        });

        let mut module = IRModule::new();
        module.add_function(func);

        let mut gen = X86_64Generator::new();
        let asm = gen.generate(&module).unwrap();
        assert!(asm.contains(".globl _start"));
        #[cfg(target_os = "macos")]
        assert!(asm.contains("_main:"));
        #[cfg(not(target_os = "macos"))]
        assert!(asm.contains("main:"));
        #[cfg(target_os = "macos")]
        assert!(asm.contains("call _main"));
        #[cfg(not(target_os = "macos"))]
        assert!(asm.contains("call main"));
    }
}
