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
        if name.starts_with("arg") {
            if let Ok(num) = name[3..].parse::<i32>() {
                return Some(16 + num * 8);
            }
        }
        self.var_map.get(name).copied()
    }

    fn is_arg_var(&self, name: &str) -> bool {
        name.starts_with("arg")
    }

    fn load_value(&self, value: &IRValue, dest_reg: &str) -> String {
        match value {
            IRValue::Const(n) => format!("    mov {}, #{}\n", dest_reg, n),
            IRValue::Var(name) => {
                if self.is_arg_var(name) {
                    if let Some(offset) = self.get_offset(name) {
                        self.generate_load(offset, dest_reg)
                    } else {
                        String::new()
                    }
                } else if let Some(offset) = self.get_offset(name) {
                    self.generate_load(offset, dest_reg)
                } else {
                    String::new()
                }
            }
        }
    }
}

impl CodeGenerator for ARM64Generator {
    fn generate(&mut self, module: &IRModule) -> Result<String, String> {
        let mut output = String::new();

        output.push_str(".text\n");
        output.push_str(".globl _main\n\n");

        for func in &module.functions {
            output.push_str(&self.generate_function(func));
        }

        output.push_str(".globl _start\n");
        output.push_str("_start:\n");
        output.push_str("    bl _main\n");
        output.push_str("    mov x16, #1\n");
        output.push_str("    svc #0x80\n");

        Ok(output)
    }
}

impl ARM64Generator {
    fn generate_function(&mut self, func: &IRFunction) -> String {
        let mut output = String::new();

        self.var_map.clear();
        self.stack_size = 0;

        output.push_str(&format!("_{}:\n", func.name));
        output.push_str("    stp x29, x30, [sp, #-16]!\n");
        output.push_str("    mov x29, sp\n");

        let arg_regs = ["x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7"];

        for instr in &func.instructions {
            if let IRInstruction::Alloc { dest, .. } = instr {
                self.alloc_stack(dest.clone());
            }
        }

        let stack_space = (self.stack_size + 31) & !31;
        if stack_space > 0 {
            output.push_str(&format!("    sub sp, sp, #{}\n\n", stack_space));
        }

        for instr in &func.instructions {
            if let IRInstruction::Alloc { dest, .. } = instr {
                if dest.starts_with("arg") {
                    if let Ok(num) = dest[3..].parse::<usize>() {
                        if num < arg_regs.len() {
                            if let Some(offset) = self.get_offset(&dest) {
                                output.push_str(&format!(
                                    "    str {}, [x29, #-{}]\n",
                                    arg_regs[num], offset
                                ));
                            }
                            continue;
                        }
                    }
                }
            }
            output.push_str(&self.generate_instruction(instr));
        }

        output.push_str("\n    mov sp, x29\n");
        output.push_str("    ldp x29, x30, [sp], #16\n");
        output.push_str("    ret\n\n");

        output
    }

    fn generate_store(&self, reg: &str, offset: i32) -> String {
        if offset <= 256 {
            format!("    str {}, [x29, #-{}]\n", reg, offset)
        } else {
            format!("    sub x17, x29, #{}\n    str {}, [x17]\n", offset, reg)
        }
    }

    fn generate_load(&self, offset: i32, reg: &str) -> String {
        if offset <= 256 {
            format!("    ldr {}, [x29, #-{}]\n", reg, offset)
        } else {
            format!("    sub x17, x29, #{}\n    ldr {}, [x17]\n", offset, reg)
        }
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
                            format!("    mov x0, #{}\n{}", n, self.generate_store("x0", offset))
                        }
                        IRValue::Var(name) => {
                            if let Some(src_offset) = self.get_offset(name) {
                                format!(
                                    "{}{}",
                                    self.generate_load(src_offset, "x0"),
                                    self.generate_store("x0", offset)
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
                            "{}{}",
                            self.generate_load(offset, "x0"),
                            self.generate_store("x0", dest_offset)
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
                                code.push_str(&self.generate_load(offset, "x0"));
                            }
                        }
                    }

                    match rhs {
                        IRValue::Const(n) => {
                            code.push_str(&format!("    add x0, x0, #{}\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&self.generate_load(offset, "x1"));
                                code.push_str("    add x0, x0, x1\n");
                            }
                        }
                    }

                    code.push_str(&self.generate_store("x0", dest_offset));
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
                            code.push_str(&format!("    mov x0, #{}\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&self.generate_load(offset, "x0"));
                            }
                        }
                    }

                    match rhs {
                        IRValue::Const(n) => {
                            code.push_str(&format!("    sub x0, x0, #{}\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&self.generate_load(offset, "x1"));
                                code.push_str("    sub x0, x0, x1\n");
                            }
                        }
                    }

                    code.push_str(&self.generate_store("x0", dest_offset));
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
                                code.push_str(&self.generate_load(offset, "x0"));
                            }
                        }
                    }

                    match rhs {
                        IRValue::Const(n) => code.push_str(&format!("    mul x0, x0, #{}\n", n)),
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&self.generate_load(offset, "x1"));
                                code.push_str("    mul x0, x0, x1\n");
                            }
                        }
                    }

                    code.push_str(&self.generate_store("x0", dest_offset));
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
                                code.push_str(&self.generate_load(offset, "x0"));
                            }
                        }
                    }

                    match rhs {
                        IRValue::Const(n) => {
                            code.push_str(&format!("    mov x1, #{}\n    sdiv x0, x0, x1\n", n));
                        }
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&self.generate_load(offset, "x1"));
                                code.push_str("    sdiv x0, x0, x1\n");
                            }
                        }
                    }

                    code.push_str(&self.generate_store("x0", dest_offset));
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
                                code.push_str(&self.generate_load(offset, "x0"));
                            }
                        }
                    }

                    match rhs {
                        IRValue::Const(n) => code.push_str(&format!("    cmp x0, #{}\n", n)),
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&self.generate_load(offset, "x1"));
                                code.push_str("    cmp x0, x1\n");
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
                    code.push_str(&self.generate_store("x0", dest_offset));
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
                                    code.push_str(&self.generate_load(offset, arg_regs[i]));
                                }
                            }
                        }
                    }
                }

                code.push_str(&format!("    bl _{}\n", function));

                if let Some(res) = result {
                    self.alloc_stack(res.clone());
                    if let Some(res_offset) = self.get_offset(res) {
                        code.push_str(&self.generate_store("x0", res_offset));
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
                            code.push_str(&self.generate_load(offset, "x0"));
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
                let mut code = String::new();

                if let Some(v) = value {
                    match v {
                        IRValue::Const(n) => code.push_str(&format!("    mov x0, #{}\n", n)),
                        IRValue::Var(name) => {
                            if let Some(offset) = self.get_offset(name) {
                                code.push_str(&self.generate_load(offset, "x0"));
                            } else {
                                code.push_str("    mov x0, #0\n");
                            }
                        }
                    }
                } else {
                    code.push_str("    mov x0, #0\n");
                }

                code.push_str("    mov sp, x29\n");
                code.push_str("    ldp x29, x30, [sp], #16\n");
                code.push_str("    ret\n");

                code
            }
        }
    }
}
