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
            IRValue::Str(_) => format!("    mov {}, #0\n", dest_reg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{CmpOp, IRInstruction, IRType};

    fn alloc(gen: &mut ARM64Generator, name: &str) {
        let _ = gen.generate_instruction(&IRInstruction::Alloc {
            dest: name.to_string(),
            ty: IRType::I64,
        });
    }

    #[test]
    fn test_helper_methods_and_offsets() {
        let mut gen = ARM64Generator::new();
        alloc(&mut gen, "x");

        assert_eq!(gen.get_offset("x"), Some(16));
        assert_eq!(gen.get_offset("arg2"), Some(32));
        assert_eq!(gen.get_offset("argx"), None);
        assert!(gen.is_arg_var("arg0"));
        assert!(!gen.is_arg_var("value"));

        assert!(gen
            .load_value(&IRValue::Const(7), "x3")
            .contains("mov x3, #7"));
        assert!(gen
            .load_value(&IRValue::Var("x".to_string()), "x3")
            .contains("ldr x3"));
        assert!(gen
            .load_value(&IRValue::Var("arg0".to_string()), "x4")
            .contains("ldr x4"));
        assert_eq!(
            gen.load_value(&IRValue::Var("argx".to_string()), "x5"),
            ""
        );
        assert_eq!(
            gen.load_value(&IRValue::Var("missing".to_string()), "x3"),
            ""
        );

        assert!(gen.generate_store("x0", 16).contains("str x0, [x29, #-16]"));
        assert!(gen.generate_store("x0", 300).contains("sub x17, x29, #300"));
        assert!(gen.generate_load(16, "x0").contains("ldr x0, [x29, #-16]"));
        assert!(gen.generate_load(300, "x0").contains("sub x17, x29, #300"));
    }

    #[test]
    fn test_generate_instruction_variants() {
        let mut gen = ARM64Generator::new();
        for name in ["a", "b", "c", "cond"] {
            alloc(&mut gen, name);
        }

        let store_const = gen.generate_instruction(&IRInstruction::Store {
            dest: "a".to_string(),
            ty: IRType::I64,
            value: IRValue::Const(1),
        });
        assert!(store_const.contains("mov x0, #1"));

        let store_var = gen.generate_instruction(&IRInstruction::Store {
            dest: "b".to_string(),
            ty: IRType::I64,
            value: IRValue::Var("a".to_string()),
        });
        assert!(store_var.contains("ldr x0"));
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
                dest: "missing_dest".to_string(),
                ty: IRType::I64,
                value: IRValue::Const(1),
            }),
            ""
        );

        let load_ok = gen.generate_instruction(&IRInstruction::Load {
            dest: "d".to_string(),
            ty: IRType::I64,
            ptr: "a".to_string(),
        });
        assert!(load_ok.contains("ldr x0"));

        let load_missing = gen.generate_instruction(&IRInstruction::Load {
            dest: "d2".to_string(),
            ty: IRType::I64,
            ptr: "missing".to_string(),
        });
        assert_eq!(load_missing, "");

        let add = gen.generate_instruction(&IRInstruction::Add {
            dest: "e".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("a".to_string()),
            rhs: IRValue::Const(2),
        });
        assert!(add.contains("add x0"));
        let add_const_lhs_var_rhs = gen.generate_instruction(&IRInstruction::Add {
            dest: "e2".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Const(4),
            rhs: IRValue::Var("a".to_string()),
        });
        assert!(add_const_lhs_var_rhs.contains("mov x0, #4"));
        assert!(add_const_lhs_var_rhs.contains("add x0, x0, x1"));

        let sub = gen.generate_instruction(&IRInstruction::Sub {
            dest: "f".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Const(8),
            rhs: IRValue::Var("b".to_string()),
        });
        assert!(sub.contains("sub x0"));
        let sub_var_lhs_const_rhs = gen.generate_instruction(&IRInstruction::Sub {
            dest: "f2".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("a".to_string()),
            rhs: IRValue::Const(1),
        });
        assert!(sub_var_lhs_const_rhs.contains("sub x0, x0, #1"));

        let mul = gen.generate_instruction(&IRInstruction::Mul {
            dest: "g".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("a".to_string()),
            rhs: IRValue::Var("b".to_string()),
        });
        assert!(mul.contains("mul x0"));

        let div = gen.generate_instruction(&IRInstruction::Div {
            dest: "h".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Const(9),
            rhs: IRValue::Const(3),
        });
        assert!(div.contains("sdiv x0"));
        let div_var_var = gen.generate_instruction(&IRInstruction::Div {
            dest: "h2".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("a".to_string()),
            rhs: IRValue::Var("b".to_string()),
        });
        assert!(div_var_var.contains("sdiv x0, x0, x1"));

        let cmp_ops = [
            (CmpOp::Eq, "cset x0, eq"),
            (CmpOp::Ne, "cset x0, ne"),
            (CmpOp::Lt, "cset x0, lt"),
            (CmpOp::Le, "cset x0, le"),
            (CmpOp::Gt, "cset x0, gt"),
            (CmpOp::Ge, "cset x0, ge"),
        ];
        for (op, needle) in cmp_ops {
            let cmp = gen.generate_instruction(&IRInstruction::Cmp {
                dest: "cmp".to_string(),
                op,
                lhs: IRValue::Const(1),
                rhs: IRValue::Var("a".to_string()),
            });
            assert!(cmp.contains(needle));
        }
        let cmp_var_const = gen.generate_instruction(&IRInstruction::Cmp {
            dest: "cmp2".to_string(),
            op: CmpOp::Eq,
            lhs: IRValue::Var("a".to_string()),
            rhs: IRValue::Const(1),
        });
        assert!(cmp_var_const.contains("cmp x0, #1"));

        let call_with_result = gen.generate_instruction(&IRInstruction::Call {
            result: Some("retv".to_string()),
            function: "foo".to_string(),
            args: vec![IRValue::Const(1), IRValue::Var("a".to_string())],
        });
        assert!(call_with_result.contains("bl _foo"));
        assert!(call_with_result.contains("str x0"));

        let call_without_result = gen.generate_instruction(&IRInstruction::Call {
            result: None,
            function: "bar".to_string(),
            args: vec![IRValue::Const(2)],
        });
        assert!(call_without_result.contains("bl _bar"));
        let many_args_call = gen.generate_instruction(&IRInstruction::Call {
            result: None,
            function: "baz".to_string(),
            args: vec![
                IRValue::Const(0),
                IRValue::Const(1),
                IRValue::Const(2),
                IRValue::Const(3),
                IRValue::Const(4),
                IRValue::Const(5),
                IRValue::Const(6),
                IRValue::Const(7),
                IRValue::Const(8),
            ],
        });
        assert!(many_args_call.contains("mov x7, #7"));
        assert!(many_args_call.contains("bl _baz"));

        assert_eq!(
            gen.generate_instruction(&IRInstruction::Label("L1".to_string())),
            "L1:\n"
        );
        assert_eq!(
            gen.generate_instruction(&IRInstruction::Jump("L2".to_string())),
            "    b L2\n"
        );

        let br_var = gen.generate_instruction(&IRInstruction::CondBranch {
            condition: IRValue::Var("cond".to_string()),
            true_label: "T".to_string(),
            false_label: "F".to_string(),
        });
        assert!(br_var.contains("b.ne T"));
        assert!(br_var.contains("b F"));

        let br_const = gen.generate_instruction(&IRInstruction::CondBranch {
            condition: IRValue::Const(1),
            true_label: "T2".to_string(),
            false_label: "F2".to_string(),
        });
        assert!(br_const.contains("cmp x0, #1"));

        let ret_const = gen.generate_instruction(&IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Const(9)),
        });
        assert!(ret_const.contains("mov x0, #9"));

        let ret_var_missing = gen.generate_instruction(&IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Var("missing".to_string())),
        });
        assert!(ret_var_missing.contains("mov x0, #0"));

        let ret_none = gen.generate_instruction(&IRInstruction::Ret {
            ty: IRType::I64,
            value: None,
        });
        assert!(ret_none.contains("mov x0, #0"));
    }

    #[test]
    fn test_generate_module_output() {
        let mut func = IRFunction::new("main".to_string(), IRType::I64);
        func.instructions.push(IRInstruction::Alloc {
            dest: "arg0".to_string(),
            ty: IRType::I64,
        });
        func.instructions.push(IRInstruction::Alloc {
            dest: "arg8".to_string(),
            ty: IRType::I64,
        });
        func.instructions.push(IRInstruction::Alloc {
            dest: "argx".to_string(),
            ty: IRType::I64,
        });
        func.instructions.push(IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Const(0)),
        });

        let mut module = IRModule::new();
        module.add_function(func);

        let mut gen = ARM64Generator::new();
        let asm = gen.generate(&module).unwrap();
        assert!(asm.contains(".globl _main"));
        assert!(asm.contains("_main:"));
        assert!(asm.contains(".globl _start"));
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
                        IRValue::Str(_) => {
                            format!("    mov x0, #0\n{}", self.generate_store("x0", offset))
                        }
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
                        "{}{}",
                        self.generate_load(offset, "x0"),
                        self.generate_store("x0", dest_offset)
                    )
                } else {
                    String::new()
                }
            }

            IRInstruction::Add { dest, lhs, rhs, .. } => {
                self.alloc_stack(dest.clone());
                let dest_offset = self.get_offset(dest).expect("allocated dest must exist");
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
                    IRValue::Str(_) => code.push_str("    mov x0, #0\n"),
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
                    IRValue::Str(_) => code.push_str("    add x0, x0, #0\n"),
                }

                code.push_str(&self.generate_store("x0", dest_offset));
                code
            }

            IRInstruction::Sub { dest, lhs, rhs, .. } => {
                self.alloc_stack(dest.clone());
                let dest_offset = self.get_offset(dest).expect("allocated dest must exist");
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
                    IRValue::Str(_) => code.push_str("    mov x0, #0\n"),
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
                    IRValue::Str(_) => code.push_str("    sub x0, x0, #0\n"),
                }

                code.push_str(&self.generate_store("x0", dest_offset));
                code
            }

            IRInstruction::Mul { dest, lhs, rhs, .. } => {
                self.alloc_stack(dest.clone());
                let dest_offset = self.get_offset(dest).expect("allocated dest must exist");
                let mut code = String::new();

                match lhs {
                    IRValue::Const(n) => code.push_str(&format!("    mov x0, #{}\n", n)),
                    IRValue::Var(name) => {
                        if let Some(offset) = self.get_offset(name) {
                            code.push_str(&self.generate_load(offset, "x0"));
                        }
                    }
                    IRValue::Str(_) => code.push_str("    mov x0, #0\n"),
                }

                match rhs {
                    IRValue::Const(n) => code.push_str(&format!("    mul x0, x0, #{}\n", n)),
                    IRValue::Var(name) => {
                        if let Some(offset) = self.get_offset(name) {
                            code.push_str(&self.generate_load(offset, "x1"));
                            code.push_str("    mul x0, x0, x1\n");
                        }
                    }
                    IRValue::Str(_) => code.push_str("    mul x0, x0, #0\n"),
                }

                code.push_str(&self.generate_store("x0", dest_offset));
                code
            }

            IRInstruction::Div { dest, lhs, rhs, .. } => {
                self.alloc_stack(dest.clone());
                let dest_offset = self.get_offset(dest).expect("allocated dest must exist");
                let mut code = String::new();

                match lhs {
                    IRValue::Const(n) => code.push_str(&format!("    mov x0, #{}\n", n)),
                    IRValue::Var(name) => {
                        if let Some(offset) = self.get_offset(name) {
                            code.push_str(&self.generate_load(offset, "x0"));
                        }
                    }
                    IRValue::Str(_) => code.push_str("    mov x0, #0\n"),
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
                    IRValue::Str(_) => code.push_str("    mov x1, #1\n    sdiv x0, x0, x1\n"),
                }

                code.push_str(&self.generate_store("x0", dest_offset));
                code
            }

            IRInstruction::Cmp { dest, op, lhs, rhs } => {
                self.alloc_stack(dest.clone());
                let dest_offset = self.get_offset(dest).expect("allocated dest must exist");
                let mut code = String::new();

                match lhs {
                    IRValue::Const(n) => code.push_str(&format!("    mov x0, #{}\n", n)),
                    IRValue::Var(name) => {
                        if let Some(offset) = self.get_offset(name) {
                            code.push_str(&self.generate_load(offset, "x0"));
                        }
                    }
                    IRValue::Str(_) => code.push_str("    mov x0, #0\n"),
                }

                match rhs {
                    IRValue::Const(n) => code.push_str(&format!("    cmp x0, #{}\n", n)),
                    IRValue::Var(name) => {
                        if let Some(offset) = self.get_offset(name) {
                            code.push_str(&self.generate_load(offset, "x1"));
                            code.push_str("    cmp x0, x1\n");
                        }
                    }
                    IRValue::Str(_) => code.push_str("    cmp x0, #0\n"),
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
                            IRValue::Str(_) => {
                                code.push_str(&format!("    mov {}, #0\n", arg_regs[i]));
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
                    IRValue::Str(_) => code.push_str("    cmp x0, #1\n"),
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
                        IRValue::Str(_) => code.push_str("    mov x0, #0\n"),
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
