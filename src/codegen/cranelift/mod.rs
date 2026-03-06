use crate::codegen::CodeGenerator;
use crate::ir::{CmpOp, IRFunction, IRInstruction, IRModule, IRType, IRValue};
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::{AbiParam, Block, InstBuilder, Signature, Type, Value};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::Context;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{default_libcall_names, Linkage, Module as CraneliftModule};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::collections::HashMap;

pub struct CraneliftGenerator {
    module: ObjectModule,
}

impl CraneliftGenerator {
    pub fn new() -> Result<Self, String> {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();

        let isa_builder = cranelift_native::builder()
            .map_err(|e| format!("Failed to create ISA builder: {}", e))?;

        let flags = settings::Flags::new(flag_builder);
        let isa = isa_builder
            .finish(flags)
            .map_err(|e| format!("Failed to create ISA: {}", e))?;

        let builder = ObjectBuilder::new(isa, "ziv", default_libcall_names())
            .map_err(|e| format!("Failed to create object builder: {}", e))?;

        Ok(CraneliftGenerator {
            module: ObjectModule::new(builder),
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

        self.module
            .finish()
            .emit()
            .map_err(|e| format!("Failed to emit object: {}", e))
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

        let func_id = self
            .module
            .declare_function(&func.name, Linkage::Export, &sig)
            .map_err(|e| format!("Failed to declare function '{}': {}", func.name, e))?;

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

        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| format!("Failed to define function '{}': {}", func.name, e))?;

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
        _ptr_type: Type,
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
                let val = self.load_value(value, builder, variables)?;
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
                let left = self.load_value(lhs, builder, variables)?;
                let right = self.load_value(rhs, builder, variables)?;
                let result = builder.ins().iadd(left, right);

                let var = Variable::from_u32(*var_counter);
                *var_counter += 1;
                builder.declare_var(var, types::I64);
                builder.def_var(var, result);
                variables.insert(dest.clone(), var);
                Ok(())
            }

            IRInstruction::Sub { dest, lhs, rhs, .. } => {
                let left = self.load_value(lhs, builder, variables)?;
                let right = self.load_value(rhs, builder, variables)?;
                let result = builder.ins().isub(left, right);

                let var = Variable::from_u32(*var_counter);
                *var_counter += 1;
                builder.declare_var(var, types::I64);
                builder.def_var(var, result);
                variables.insert(dest.clone(), var);
                Ok(())
            }

            IRInstruction::Mul { dest, lhs, rhs, .. } => {
                let left = self.load_value(lhs, builder, variables)?;
                let right = self.load_value(rhs, builder, variables)?;
                let result = builder.ins().imul(left, right);

                let var = Variable::from_u32(*var_counter);
                *var_counter += 1;
                builder.declare_var(var, types::I64);
                builder.def_var(var, result);
                variables.insert(dest.clone(), var);
                Ok(())
            }

            IRInstruction::Div { dest, lhs, rhs, .. } => {
                let left = self.load_value(lhs, builder, variables)?;
                let right = self.load_value(rhs, builder, variables)?;
                let result = builder.ins().sdiv(left, right);

                let var = Variable::from_u32(*var_counter);
                *var_counter += 1;
                builder.declare_var(var, types::I64);
                builder.def_var(var, result);
                variables.insert(dest.clone(), var);
                Ok(())
            }

            IRInstruction::Cmp { dest, op, lhs, rhs } => {
                let left = self.load_value(lhs, builder, variables)?;
                let right = self.load_value(rhs, builder, variables)?;

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
                let cond_val = self.load_value(condition, builder, variables)?;
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
                    let val = self.load_value(v, builder, variables)?;
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
                    .map(|arg| self.load_value(arg, builder, variables))
                    .collect::<Result<Vec<_>, _>>()?;

                let mut sig = Signature::new(CallConv::SystemV);
                for _ in args {
                    sig.params.push(AbiParam::new(types::I64));
                }
                sig.returns.push(AbiParam::new(types::I64));

                // Try to get existing function declaration first
                let func_id = if let Some(cranelift_module::FuncOrDataId::Func(id)) = self.module.get_name(function) {
                    id
                } else {
                    self.module
                        .declare_function(function, Linkage::Import, &sig)
                        .map_err(|e| format!("Failed to declare function '{}': {}", function, e))?
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
        &self,
        value: &IRValue,
        builder: &mut FunctionBuilder,
        variables: &HashMap<String, Variable>,
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
