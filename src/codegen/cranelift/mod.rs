use crate::codegen::CodeGenerator;
use crate::ir::{CmpOp, IRFunction, IRInstruction, IRModule, IRType, IRValue};
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::{AbiParam, Block, InstBuilder, Signature, Type, Value};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::Context;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{
    default_libcall_names, DataDescription, DataId, Linkage, Module as CraneliftModule,
};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::collections::HashMap;
use std::fmt::Display;

pub struct CraneliftGenerator {
    module: ObjectModule,
    string_data_ids: HashMap<String, DataId>,
}

fn with_context<T, E: Display>(result: Result<T, E>, context: &str) -> Result<T, String> {
    result.map_err(|e| format!("{}: {}", context, e))
}

impl CraneliftGenerator {
    pub fn new() -> Result<Self, String> {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "true").unwrap();

        let isa_builder = with_context(cranelift_native::builder(), "Failed to create ISA builder")?;

        let flags = settings::Flags::new(flag_builder);
        let isa = with_context(isa_builder.finish(flags), "Failed to create ISA")?;

        let builder = with_context(
            ObjectBuilder::new(isa, "ziv", default_libcall_names()),
            "Failed to create object builder",
        )?;

        Ok(CraneliftGenerator {
            module: ObjectModule::new(builder),
            string_data_ids: HashMap::new(),
        })
    }

    fn translate_type(ty: &IRType) -> Option<Type> {
        match ty {
            IRType::I64 => Some(types::I64),
            IRType::Void => None,
        }
    }

    pub fn compile_to_object(mut self, module: &IRModule) -> Result<Vec<u8>, String> {
        for func in &module.functions {
            self.compile_function(func)?;
        }

        with_context(self.module.finish().emit(), "Failed to emit object")
    }

    fn intern_string_data(&mut self, value: &str) -> Result<DataId, String> {
        if let Some(id) = self.string_data_ids.get(value).copied() {
            return Ok(id);
        }

        let symbol_name = format!("ziv_str_{}", self.string_data_ids.len());
        let data_id = with_context(
            self.module
                .declare_data(&symbol_name, Linkage::Local, false, false),
            &format!("Failed to declare data '{}'", symbol_name),
        )?;

        let mut description = DataDescription::new();
        let mut bytes = value.as_bytes().to_vec();
        bytes.push(0);
        description.define(bytes.into_boxed_slice());

        with_context(
            self.module.define_data(data_id, &description),
            &format!("Failed to define data '{}'", symbol_name),
        )?;

        self.string_data_ids.insert(value.to_string(), data_id);
        Ok(data_id)
    }

    fn compile_function(&mut self, func: &IRFunction) -> Result<(), String> {
        let ptr_type = Type::int_with_byte_size(8).unwrap();

        let mut sig = Signature::new(CallConv::SystemV);
        for _ in &func.params {
            sig.params.push(AbiParam::new(types::I64));
        }
        if let Some(ret_type) = Self::translate_type(&func.ret_ty) {
            sig.returns.push(AbiParam::new(ret_type));
        }

        let func_id = with_context(
            self.module
                .declare_function(&func.name, Linkage::Export, &sig),
            &format!("Failed to declare function '{}'", func.name),
        )?;

        let mut ctx = Context::new();
        ctx.func.signature = sig;

        let mut func_ctx = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            let mut variables: HashMap<String, Variable> = HashMap::new();
            let mut var_counter = 0u32;
            let mut block_map: HashMap<String, Block> = HashMap::new();

            for (i, (param_name, _)) in func.params.iter().enumerate() {
                let param_value = builder.block_params(entry_block)[i];
                let var = Variable::from_u32(var_counter);
                var_counter += 1;
                builder.declare_var(var, types::I64);
                builder.def_var(var, param_value);
                variables.insert(param_name.clone(), var);
            }

            for instr in &func.instructions {
                self.translate_instruction(
                    instr,
                    &mut builder,
                    &mut variables,
                    &mut var_counter,
                    &mut block_map,
                    Self::translate_type(&func.ret_ty),
                    ptr_type,
                )?;
            }

            // Seal all blocks before finalizing
            for (_, block) in block_map.iter() {
                builder.seal_block(*block);
            }

            builder.finalize();
        }

        with_context(
            self.module.define_function(func_id, &mut ctx),
            &format!("Failed to define function '{}'", func.name),
        )?;

        ctx.clear();
        Ok(())
    }

    fn translate_instruction(
        &mut self,
        instr: &IRInstruction,
        builder: &mut FunctionBuilder,
        variables: &mut HashMap<String, Variable>,
        var_counter: &mut u32,
        block_map: &mut HashMap<String, Block>,
        _ret_type: Option<Type>,
        ptr_type: Type,
    ) -> Result<(), String> {
        match instr {
            IRInstruction::Alloc { dest, .. } => {
                // Only allocate if not already a parameter
                if !variables.contains_key(dest) {
                    let var = Variable::from_u32(*var_counter);
                    *var_counter += 1;
                    builder.declare_var(var, types::I64);
                    variables.insert(dest.clone(), var);
                }
                Ok(())
            }

            IRInstruction::Store { dest, value, .. } => {
                let val = self.load_value(value, builder, variables, ptr_type)?;
                if let Some(var) = variables.get(dest) {
                    builder.def_var(*var, val);
                }
                Ok(())
            }

            IRInstruction::Load { dest, ptr, .. } => {
                if let Some(src_var) = variables.get(ptr) {
                    let val = builder.use_var(*src_var);
                    let dest_var = Variable::from_u32(*var_counter);
                    *var_counter += 1;
                    builder.declare_var(dest_var, types::I64);
                    builder.def_var(dest_var, val);
                    variables.insert(dest.clone(), dest_var);
                }
                Ok(())
            }

            IRInstruction::Add { dest, lhs, rhs, .. } => {
                let left = self.load_value(lhs, builder, variables, ptr_type)?;
                let right = self.load_value(rhs, builder, variables, ptr_type)?;
                let result = builder.ins().iadd(left, right);

                let var = Variable::from_u32(*var_counter);
                *var_counter += 1;
                builder.declare_var(var, types::I64);
                builder.def_var(var, result);
                variables.insert(dest.clone(), var);
                Ok(())
            }

            IRInstruction::Sub { dest, lhs, rhs, .. } => {
                let left = self.load_value(lhs, builder, variables, ptr_type)?;
                let right = self.load_value(rhs, builder, variables, ptr_type)?;
                let result = builder.ins().isub(left, right);

                let var = Variable::from_u32(*var_counter);
                *var_counter += 1;
                builder.declare_var(var, types::I64);
                builder.def_var(var, result);
                variables.insert(dest.clone(), var);
                Ok(())
            }

            IRInstruction::Mul { dest, lhs, rhs, .. } => {
                let left = self.load_value(lhs, builder, variables, ptr_type)?;
                let right = self.load_value(rhs, builder, variables, ptr_type)?;
                let result = builder.ins().imul(left, right);

                let var = Variable::from_u32(*var_counter);
                *var_counter += 1;
                builder.declare_var(var, types::I64);
                builder.def_var(var, result);
                variables.insert(dest.clone(), var);
                Ok(())
            }

            IRInstruction::Div { dest, lhs, rhs, .. } => {
                let left = self.load_value(lhs, builder, variables, ptr_type)?;
                let right = self.load_value(rhs, builder, variables, ptr_type)?;
                let result = builder.ins().sdiv(left, right);

                let var = Variable::from_u32(*var_counter);
                *var_counter += 1;
                builder.declare_var(var, types::I64);
                builder.def_var(var, result);
                variables.insert(dest.clone(), var);
                Ok(())
            }

            IRInstruction::Cmp { dest, op, lhs, rhs } => {
                let left = self.load_value(lhs, builder, variables, ptr_type)?;
                let right = self.load_value(rhs, builder, variables, ptr_type)?;

                let cond = match op {
                    CmpOp::Eq => IntCC::Equal,
                    CmpOp::Ne => IntCC::NotEqual,
                    CmpOp::Lt => IntCC::SignedLessThan,
                    CmpOp::Le => IntCC::SignedLessThanOrEqual,
                    CmpOp::Gt => IntCC::SignedGreaterThan,
                    CmpOp::Ge => IntCC::SignedGreaterThanOrEqual,
                };

                let cmp_result = builder.ins().icmp(cond, left, right);
                let result = builder.ins().uextend(types::I64, cmp_result);

                let var = Variable::from_u32(*var_counter);
                *var_counter += 1;
                builder.declare_var(var, types::I64);
                builder.def_var(var, result);
                variables.insert(dest.clone(), var);
                Ok(())
            }

            IRInstruction::Label(name) => {
                // Get or create the block
                let block = block_map.entry(name.clone()).or_insert_with(|| builder.create_block());
                // Switch to this block
                builder.switch_to_block(*block);
                Ok(())
            }

            IRInstruction::Jump(label) => {
                // Get or create the target block
                let block = block_map.entry(label.clone()).or_insert_with(|| builder.create_block());
                builder.ins().jump(*block, &[]);
                Ok(())
            }

            IRInstruction::CondBranch {
                condition,
                true_label,
                false_label,
            } => {
                let cond_val = self.load_value(condition, builder, variables, ptr_type)?;
                let zero = builder.ins().iconst(types::I64, 0);
                let cmp = builder.ins().icmp(IntCC::NotEqual, cond_val, zero);

                // Create blocks for targets if they don't exist
                let true_block = *block_map.entry(true_label.clone()).or_insert_with(|| builder.create_block());
                let false_block = *block_map.entry(false_label.clone()).or_insert_with(|| builder.create_block());

                builder.ins().brif(cmp, true_block, &[], false_block, &[]);
                Ok(())
            }

            IRInstruction::Ret { value, .. } => {
                if let Some(v) = value {
                    let val = self.load_value(v, builder, variables, ptr_type)?;
                    builder.ins().return_(&[val]);
                } else {
                    builder.ins().return_(&[]);
                }
                Ok(())
            }

            IRInstruction::Call {
                result,
                function,
                args,
            } => {
                let arg_values: Vec<Value> = args
                    .iter()
                    .map(|arg| self.load_value(arg, builder, variables, ptr_type))
                    .collect::<Result<Vec<_>, _>>()?;

                let mut sig = Signature::new(CallConv::SystemV);
                for arg in args {
                    match arg {
                        IRValue::Str(_) => sig.params.push(AbiParam::new(ptr_type)),
                        _ => sig.params.push(AbiParam::new(types::I64)),
                    }
                }
                sig.returns.push(AbiParam::new(types::I64));

                // Try to get existing function declaration first
                let func_id = if let Some(cranelift_module::FuncOrDataId::Func(id)) = self.module.get_name(function) {
                    id
                } else {
                    with_context(
                        self.module.declare_function(function, Linkage::Import, &sig),
                        &format!("Failed to declare function '{}'", function),
                    )?
                };

                let func_ref = self.module.declare_func_in_func(func_id, builder.func);
                let call_result = builder.ins().call(func_ref, &arg_values);

                if let Some(dest) = result {
                    let val = builder.inst_results(call_result)[0];
                    let var = Variable::from_u32(*var_counter);
                    *var_counter += 1;
                    builder.declare_var(var, types::I64);
                    builder.def_var(var, val);
                    variables.insert(dest.clone(), var);
                }
                Ok(())
            }
        }
    }

    fn load_value(
        &mut self,
        value: &IRValue,
        builder: &mut FunctionBuilder,
        variables: &HashMap<String, Variable>,
        ptr_type: Type,
    ) -> Result<Value, String> {
        match value {
            IRValue::Const(n) => Ok(builder.ins().iconst(types::I64, *n)),
            IRValue::Var(name) => {
                if let Some(var) = variables.get(name) {
                    Ok(builder.use_var(*var))
                } else {
                    Err(format!("Undefined variable: {}", name))
                }
            }
            IRValue::Str(s) => {
                let data_id = self.intern_string_data(s)?;
                let local_id = self.module.declare_data_in_func(data_id, builder.func);
                Ok(builder.ins().symbol_value(ptr_type, local_id))
            }
        }
    }
}

impl CodeGenerator for CraneliftGenerator {
    fn generate(&mut self, module: &IRModule) -> Result<String, String> {
        for func in &module.functions {
            self.compile_function(func)?;
        }
        Ok(format!(
            "; Generated {} functions with Cranelift",
            module.functions.len()
        ))
    }
}

impl Default for CraneliftGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default CraneliftGenerator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rich_module() -> IRModule {
        let mut callee = IRFunction::new("callee".to_string(), IRType::I64);
        callee.params.push(("arg0".to_string(), IRType::I64));
        callee.instructions.push(IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Var("arg0".to_string())),
        });

        let mut main = IRFunction::new("main".to_string(), IRType::I64);
        main.instructions.push(IRInstruction::Alloc {
            dest: "a".to_string(),
            ty: IRType::I64,
        });
        main.instructions.push(IRInstruction::Store {
            dest: "a".to_string(),
            ty: IRType::I64,
            value: IRValue::Const(2),
        });
        main.instructions.push(IRInstruction::Load {
            dest: "b".to_string(),
            ty: IRType::I64,
            ptr: "a".to_string(),
        });
        main.instructions.push(IRInstruction::Add {
            dest: "c".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("b".to_string()),
            rhs: IRValue::Const(3),
        });
        main.instructions.push(IRInstruction::Sub {
            dest: "d".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("c".to_string()),
            rhs: IRValue::Const(1),
        });
        main.instructions.push(IRInstruction::Mul {
            dest: "e".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("d".to_string()),
            rhs: IRValue::Const(2),
        });
        main.instructions.push(IRInstruction::Div {
            dest: "f".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("e".to_string()),
            rhs: IRValue::Const(2),
        });
        main.instructions.push(IRInstruction::Cmp {
            dest: "cmp".to_string(),
            op: CmpOp::Eq,
            lhs: IRValue::Var("f".to_string()),
            rhs: IRValue::Const(4),
        });
        main.instructions.push(IRInstruction::Call {
            result: Some("ret1".to_string()),
            function: "callee".to_string(),
            args: vec![IRValue::Var("f".to_string())],
        });
        main.instructions.push(IRInstruction::Call {
            result: None,
            function: "ext_fn".to_string(),
            args: vec![IRValue::Const(0)],
        });
        main.instructions.push(IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Var("f".to_string())),
        });

        let mut module = IRModule::new();
        module.add_function(callee);
        module.add_function(main);
        module
    }

    #[test]
    fn test_translate_type() {
        assert_eq!(CraneliftGenerator::translate_type(&IRType::I64), Some(types::I64));
        assert_eq!(CraneliftGenerator::translate_type(&IRType::Void), None);
    }

    #[test]
    fn test_generate_and_compile_to_object() {
        let module = rich_module();
        let mut gen = CraneliftGenerator::new().unwrap();
        let text = gen.generate(&module).unwrap();
        assert!(text.contains("Generated 2 functions"));

        let obj = CraneliftGenerator::new()
            .unwrap()
            .compile_to_object(&module)
            .unwrap();
        assert!(!obj.is_empty());
    }

    #[test]
    fn test_compile_void_function() {
        let mut func = IRFunction::new("vmain".to_string(), IRType::Void);
        func.instructions.push(IRInstruction::Ret {
            ty: IRType::Void,
            value: None,
        });
        let mut module = IRModule::new();
        module.add_function(func);

        let obj = CraneliftGenerator::new()
            .unwrap()
            .compile_to_object(&module)
            .unwrap();
        assert!(!obj.is_empty());
    }

    #[test]
    fn test_compile_undefined_variable_error() {
        let mut func = IRFunction::new("bad".to_string(), IRType::I64);
        func.instructions.push(IRInstruction::Add {
            dest: "x".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("missing".to_string()),
            rhs: IRValue::Const(1),
        });
        func.instructions.push(IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Const(0)),
        });

        let mut module = IRModule::new();
        module.add_function(func);

        let err = CraneliftGenerator::new()
            .unwrap()
            .compile_to_object(&module)
            .unwrap_err();
        assert!(err.contains("Undefined variable: missing"));
    }

    #[test]
    fn test_compile_with_labels_and_branches() {
        let mut func = IRFunction::new("branched".to_string(), IRType::I64);
        func.instructions.push(IRInstruction::Alloc {
            dest: "cond".to_string(),
            ty: IRType::I64,
        });
        func.instructions.push(IRInstruction::Store {
            dest: "cond".to_string(),
            ty: IRType::I64,
            value: IRValue::Const(1),
        });
        func.instructions.push(IRInstruction::CondBranch {
            condition: IRValue::Var("cond".to_string()),
            true_label: "t".to_string(),
            false_label: "f".to_string(),
        });
        func.instructions.push(IRInstruction::Label("t".to_string()));
        func.instructions.push(IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Const(1)),
        });
        func.instructions.push(IRInstruction::Label("f".to_string()));
        func.instructions.push(IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Const(0)),
        });

        let mut module = IRModule::new();
        module.add_function(func);

        let obj = CraneliftGenerator::new()
            .unwrap()
            .compile_to_object(&module)
            .unwrap();
        assert!(!obj.is_empty());
    }

    #[test]
    fn test_compile_cmp_variants_and_jump() {
        let mut func = IRFunction::new("cmp_jump".to_string(), IRType::I64);
        func.instructions.push(IRInstruction::Alloc {
            dest: "a".to_string(),
            ty: IRType::I64,
        });
        func.instructions.push(IRInstruction::Store {
            dest: "a".to_string(),
            ty: IRType::I64,
            value: IRValue::Const(3),
        });

        for (i, op) in [
            CmpOp::Ne,
            CmpOp::Lt,
            CmpOp::Le,
            CmpOp::Gt,
            CmpOp::Ge,
        ]
        .into_iter()
        .enumerate()
        {
            func.instructions.push(IRInstruction::Cmp {
                dest: format!("cmp{}", i),
                op,
                lhs: IRValue::Var("a".to_string()),
                rhs: IRValue::Const(i as i64),
            });
        }

        func.instructions.push(IRInstruction::Jump("end".to_string()));
        func.instructions.push(IRInstruction::Label("end".to_string()));
        func.instructions.push(IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Const(0)),
        });

        let mut module = IRModule::new();
        module.add_function(func);

        let obj = CraneliftGenerator::new()
            .unwrap()
            .compile_to_object(&module)
            .unwrap();
        assert!(!obj.is_empty());
    }

    #[test]
    fn test_with_context_helper() {
        let ok: Result<i32, &str> = Ok(7);
        assert_eq!(with_context(ok, "ctx").unwrap(), 7);

        let err: Result<i32, &str> = Err("boom");
        assert_eq!(with_context(err, "ctx").unwrap_err(), "ctx: boom");
    }

    #[test]
    fn test_generate_propagates_compile_error() {
        let mut func = IRFunction::new("bad_generate".to_string(), IRType::I64);
        func.instructions.push(IRInstruction::Add {
            dest: "x".to_string(),
            ty: IRType::I64,
            lhs: IRValue::Var("missing".to_string()),
            rhs: IRValue::Const(1),
        });
        func.instructions.push(IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Const(0)),
        });
        let mut module = IRModule::new();
        module.add_function(func);

        let mut gen = CraneliftGenerator::new().unwrap();
        let err = gen.generate(&module).unwrap_err();
        assert!(err.contains("Undefined variable: missing"));
    }

    #[test]
    fn test_compile_call_arg_undefined_variable_error() {
        let mut func = IRFunction::new("bad_call".to_string(), IRType::I64);
        func.instructions.push(IRInstruction::Call {
            result: Some("r".to_string()),
            function: "ext_fn".to_string(),
            args: vec![IRValue::Var("missing".to_string())],
        });
        func.instructions.push(IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Const(0)),
        });

        let mut module = IRModule::new();
        module.add_function(func);

        let err = CraneliftGenerator::new()
            .unwrap()
            .compile_to_object(&module)
            .unwrap_err();
        assert!(err.contains("Undefined variable: missing"));
    }
}
