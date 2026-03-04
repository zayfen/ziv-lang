        let mut block_map: HashMap<String, Block> = HashMap::new();

        for instr in &func.instructions {
            match instr {
                IRInstruction::Alloc { dest, .. } => {
                    let var = Variable::from_u32(*var_counter);
                    *var_counter += 1;
                    builder.declare_var(var, types::I64);
                    variables.insert(dest.clone(), var);
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
                    let block = block_map.entry(name.clone()).or_insert_with(|| builder.create_block());
                    builder.switch_to_block(*block);
                    if !builder.is_block_sealed(*block) {
                        builder.seal_block(*block);
                    }
                    Ok(())
                }

                IRInstruction::Jump(label) => {
                    let block = block_map.entry(label.clone()).or_insert_with(|| builder.create_block());
                    builder.ins().jump(*block, &[]);
                    Ok(())
                }

                IRInstruction::CondBranch { condition, true_label, false_label } => {
                    let cond_val = self.load_value(condition, builder, variables)?;
                    let zero = builder.ins().iconst(types::I64, 0);
                    let cmp = builder.ins().icmp(IntCC::NotEqual, cond_val, zero);

                    let true_block = block_map.entry(true_label.clone()).or_insert_with(|| builder.create_block());
                    let false_block = block_map.entry(false_label.clone()).or_insert_with(|| builder.create_block());
                    builder.ins().brif(cmp, *true_block, &[], *false_block, &[]);
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

                IRInstruction::Call { result, function, args } => {
                    let arg_values: Vec<Value> = args
                        .iter()
                        .map(|arg| self.load_value(arg, builder, variables))
                        .collect::<Result<Vec<_>, _>>()?;

                    let mut sig = Signature::new(CallConv::SystemV);
                    for _ in args {
                        sig.params.push(AbiParam::new(types::I64));
                    }
                    sig.returns.push(AbiParam::new(types::I64));

                    let func_id = self.module
                        .declare_function(function, Linkage::Import, &sig)
                        .map_err(|e| format!("Failed to declare function '{}': {}", function, e))?;

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

        builder.finalize();
    }

    self.module
        .define_function(func_id, &mut ctx)
        .map_err(|e| format!("Failed to define function '{}': {}", func.name, e))?;

    ctx.clear();

    Ok(())
}

ENDOFFILE
