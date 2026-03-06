# Cranelift Code Generator Migration Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace LightLang's custom x86-64 assembly generator with Cranelift for production-quality native code generation.

**Architecture:** Create a `CraneliftGenerator` that implements the existing `CodeGenerator` trait. The generator translates LightLang IR to Cranelift IR, then uses Cranelift's built-in register allocator and object file emitter to produce native code. This eliminates the external assembler dependency and provides multi-architecture support.

**Tech Stack:** Rust, Cranelift (cranelift-codegen, cranelift-module, cranelift-object), target-lexicon

---

## Current State Analysis

### IR Instructions to Translate

| Category | Instructions | Cranelift Mapping |
|----------|--------------|-------------------|
| Memory | `Alloc`, `Store`, `Load` | Stack slots + loads/stores |
| Arithmetic | `Add`, `Sub`, `Mul`, `Div` | `iadd`, `isub`, `imul`, `sdiv` |
| Comparison | `Cmp` (Eq, Ne, Lt, Le, Gt, Ge) | `icmp` with condition codes |
| Control Flow | `Label`, `Jump`, `CondBranch`, `Ret` | Blocks + branches + return |
| Function | `Call` | Direct function calls |

### Current Codegen Flow
```
IRModule → X86_64Generator::generate() → Assembly String → as → .o → clang → executable
```

### Target Cranelift Flow
```
IRModule → CraneliftGenerator::generate() → Object Bytes → linker → executable
```

---

## Task 1: Add Cranelift Dependencies

**Files:**
- Modify: `Cargo.toml:9-33`

**Step 1: Add Cranelift dependencies to Cargo.toml**

```toml
# Cranelift code generator
cranelift-codegen = "0.116"
cranelift-module = "0.116"
cranelift-object = "0.116"
target-lexicon = "0.13"
```

Add these lines after line 32 (after `scroll = "0.12"`).

**Step 2: Verify dependencies compile**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "feat: add Cranelift codegen dependencies"
```

---

## Task 2: Create Cranelift Generator Module

**Files:**
- Create: `src/codegen/cranelift.rs`
- Modify: `src/codegen/mod.rs:1-29`

**Step 1: Create the Cranelift generator stub**

Create file `src/codegen/cranelift.rs`:

```rust
//! Cranelift-based code generator for LightLang

use crate::codegen::CodeGenerator;
use crate::ir::{CmpOp, IRFunction, IRInstruction, IRModule, IRType, IRValue};
use cranelift::prelude::*;
use cranelift_module::{DataDescription, Linkage, Module};
use cranelift_object::ObjectModule;
use std::collections::HashMap;
use target_lexicon::Triple;

/// Cranelift code generator
pub struct CraneliftGenerator {
    module: ObjectModule,
    ptr_type: Type,
}

impl CraneliftGenerator {
    /// Create a new Cranelift generator for the host target
    pub fn new() -> Result<Self, String> {
        let triple = Triple::host()
            .map_err(|e| format!("Failed to get host triple: {}", e))?;
        
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        
        let isa_builder = cranelift_native::builder()
            .map_err(|e| format!("Failed to create ISA builder: {}", e))?;
        
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| format!("Failed to create ISA: {}", e))?;
        
        let mut module = ObjectModule::new(cranelift_module::default_libcall_names());
        module.set_isa(Box::new(isa));
        
        let ptr_type = Type::int_with_byte_size(8).unwrap();
        
        Ok(CraneliftGenerator { module, ptr_type })
    }
}

impl CodeGenerator for CraneliftGenerator {
    fn generate(&mut self, module: &IRModule) -> Result<String, String> {
        // For Cranelift, we don't generate assembly text.
        // Instead, we'll emit an object file directly.
        // Return a placeholder - actual implementation in emit_object()
        Ok("; Cranelift generates object files, not assembly".to_string())
    }
}

impl CraneliftGenerator {
    /// Generate and return the object file bytes
    pub fn emit_object(self) -> Result<Vec<u8>, String> {
        self.module
            .finish()
            .emit()
            .map_err(|e| format!("Failed to emit object: {}", e))
    }
}

impl Default for CraneliftGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default CraneliftGenerator")
    }
}
```

**Step 2: Update codegen module to export Cranelift**

Modify `src/codegen/mod.rs`:

```rust
//! Code generation for LightLang

pub mod arm64;
pub mod cranelift;
pub mod llvm_text;
pub mod x86_64;

use crate::ir::IRModule;

pub type CodeGenResult<T> = Result<T, String>;

/// Code generator trait
pub trait CodeGenerator {
    fn generate(&mut self, module: &IRModule) -> CodeGenResult<String>;
}

// Re-export generators
pub use arm64::ARM64Generator;
pub use cranelift::CraneliftGenerator;
pub use llvm_text::LLVMTextGenerator;
pub use x86_64::X86_64Generator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_module() {
        assert!(true);
    }
}
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/codegen/cranelift.rs src/codegen/mod.rs
git commit -m "feat: add Cranelift generator module stub"
```

---

## Task 3: Implement IR Type Translation

**Files:**
- Modify: `src/codegen/cranelift.rs:1-70`

**Step 1: Add type translation helper**

Add this method to `CraneliftGenerator` impl block:

```rust
impl CraneliftGenerator {
    // ... existing new() method ...
    
    /// Translate IR type to Cranelift type
    fn translate_type(ty: &IRType) -> Type {
        match ty {
            IRType::I64 => types::I64,
            IRType::Void => types::VOID,
        }
    }
}
```

**Step 2: Write test for type translation**

Create test at end of `src/codegen/cranelift.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use cranelift::prelude::types;

    #[test]
    fn test_type_translation() {
        assert_eq!(CraneliftGenerator::translate_type(&IRType::I64), types::I64);
        assert_eq!(CraneliftGenerator::translate_type(&IRType::Void), types::VOID);
    }
}
```

**Step 3: Run test to verify**

Run: `cargo test cranelift::tests::test_type_translation`
Expected: PASS

**Step 4: Commit**

```bash
git add src/codegen/cranelift.rs
git commit -m "feat: implement IR type translation for Cranelift"
```

---

## Task 4: Implement Function Context

**Files:**
- Modify: `src/codegen/cranelift.rs`

**Step 1: Add function context struct**

Add after imports, before `CraneliftGenerator`:

```rust
/// Context for translating a single function
struct FunctionContext<'a> {
    func_ctx: FunctionBuilderContext,
    module: &'a mut ObjectModule,
    ptr_type: Type,
    /// Map IR variable names to Cranelift SSA values
    variables: HashMap<String, Variable>,
    /// Map IR variable names to stack slots (for Alloc instructions)
    stack_slots: HashMap<String, StackSlot>,
}
```

**Step 2: Add function context methods**

Add impl block for FunctionContext:

```rust
impl<'a> FunctionContext<'a> {
    fn new(module: &'a mut ObjectModule, ptr_type: Type) -> Self {
        FunctionContext {
            func_ctx: FunctionBuilderContext::new(),
            module,
            ptr_type,
            variables: HashMap::new(),
            stack_slots: HashMap::new(),
        }
    }
    
    /// Declare a function in the module
    fn declare_function(&mut self, name: &str, sig: Signature) -> Result<FuncId, String> {
        self.module
            .declare_function(name, Linkage::Export, &sig)
            .map_err(|e| format!("Failed to declare function '{}': {}", name, e))
    }
    
    /// Define a declared function
    fn define_function(&mut self, id: FuncId, ctx: &mut Context) -> Result<(), String> {
        self.module
            .define_function(id, ctx)
            .map_err(|e| format!("Failed to define function: {}", e))
    }
}
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/codegen/cranelift.rs
git commit -m "feat: add function context for Cranelift translation"
```

---

## Task 5: Implement Function Declaration and Definition

**Files:**
- Modify: `src/codegen/cranelift.rs`

**Step 1: Add function compilation method to CraneliftGenerator**

Add to `impl CraneliftGenerator`:

```rust
impl CraneliftGenerator {
    // ... existing methods ...
    
    /// Compile a single IR function to Cranelift
    fn compile_function(&mut self, func: &IRFunction) -> Result<(), String> {
        // Create function signature
        let mut sig = self.module.target_config().default_signature();
        
        // Add parameters
        for (_, param_ty) in &func.params {
            sig.params.push(AbiParam::new(Self::translate_type(param_ty)));
        }
        
        // Set return type
        let ret_type = Self::translate_type(&func.ret_ty);
        if ret_type != types::VOID {
            sig.returns.push(AbiParam::new(ret_type));
        }
        
        // Declare the function
        let func_id = self.module
            .declare_function(&func.name, Linkage::Export, &sig)
            .map_err(|e| format!("Failed to declare function '{}': {}", func.name, e))?;
        
        // Create function context
        let mut ctx = Context::new();
        ctx.func.signature = sig;
        
        // Build function body
        let mut func_ctx = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
            
            // Create entry block
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);
            
            // Variable tracking
            let mut variables: HashMap<String, Variable> = HashMap::new();
            let mut var_counter = 0;
            
            // Map parameters to variables
            for (i, (param_name, _)) in func.params.iter().enumerate() {
                let param_value = builder.block_params(entry_block)[i];
                let var = Variable::new(var_counter);
                var_counter += 1;
                builder.declare_var(var, types::I64);
                builder.def_var(var, param_value);
                variables.insert(param_name.clone(), var);
            }
            
            // Translate instructions
            let mut block_map: HashMap<String, Block> = HashMap::new();
            for instr in &func.instructions {
                self.translate_instruction(
                    instr,
                    &mut builder,
                    &mut variables,
                    &mut var_counter,
                    &mut block_map,
                    &ret_type,
                )?;
            }
            
            builder.finalize();
        }
        
        // Define the function in the module
        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| format!("Failed to define function '{}': {}", func.name, e))?;
        
        ctx.clear();
        
        Ok(())
    }
}
```

**Step 2: Add placeholder translate_instruction method**

```rust
impl CraneliftGenerator {
    // ... existing methods ...
    
    fn translate_instruction(
        &mut self,
        instr: &IRInstruction,
        builder: &mut FunctionBuilder,
        variables: &mut HashMap<String, Variable>,
        var_counter: &mut usize,
        block_map: &mut HashMap<String, Block>,
        ret_type: &Type,
    ) -> Result<(), String> {
        // Placeholder - will implement in next tasks
        match instr {
            IRInstruction::Ret { value, .. } => {
                if let Some(v) = value {
                    let val = self.load_value(v, builder, variables)?;
                    builder.ins().return_(&[val]);
                } else {
                    builder.ins().return_(&[]);
                }
                Ok(())
            }
            _ => Ok(()) // Placeholder for other instructions
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
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/codegen/cranelift.rs
git commit -m "feat: implement function declaration and definition in Cranelift"
```

---

## Task 6: Implement Memory Instructions (Alloc, Store, Load)

**Files:**
- Modify: `src/codegen/cranelift.rs`

**Step 1: Update translate_instruction for memory ops**

Update the `translate_instruction` method to handle Alloc, Store, Load:

```rust
fn translate_instruction(
    &mut self,
    instr: &IRInstruction,
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, Variable>,
    var_counter: &mut usize,
    block_map: &mut HashMap<String, Block>,
    ret_type: &Type,
) -> Result<(), String> {
    match instr {
        IRInstruction::Alloc { dest, ty } => {
            // Create a stack slot for the variable
            let slot = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                8, // i64 = 8 bytes
                0,
            ));
            
            // Store slot for later use
            // We'll use variables map with a special prefix for stack slots
            let ptr_var = Variable::new(*var_counter);
            *var_counter += 1;
            builder.declare_var(ptr_var, self.ptr_type);
            
            // Create stack address
            let addr = builder.ins().stack_addr(self.ptr_type, slot, 0);
            builder.def_var(ptr_var, addr);
            variables.insert(format!("__slot__{}", dest), ptr_var);
            
            // Also create an SSA variable for the value itself
            let val_var = Variable::new(*var_counter);
            *var_counter += 1;
            builder.declare_var(val_var, Self::translate_type(ty));
            variables.insert(dest.clone(), val_var);
            
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
                
                let dest_var = Variable::new(*var_counter);
                *var_counter += 1;
                builder.declare_var(dest_var, types::I64);
                builder.def_var(dest_var, val);
                variables.insert(dest.clone(), dest_var);
            }
            
            Ok(())
        }
        
        // ... keep existing Ret handler ...
        IRInstruction::Ret { value, .. } => {
            if let Some(v) = value {
                let val = self.load_value(v, builder, variables)?;
                builder.ins().return_(&[val]);
            } else {
                builder.ins().return_(&[]);
            }
            Ok(())
        }
        
        _ => Ok(()) // Placeholder for remaining instructions
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/codegen/cranelift.rs
git commit -m "feat: implement memory instructions (Alloc, Store, Load) for Cranelift"
```

---

## Task 7: Implement Arithmetic Instructions

**Files:**
- Modify: `src/codegen/cranelift.rs`

**Step 1: Add arithmetic instruction handling**

Add cases to `translate_instruction`:

```rust
IRInstruction::Add { dest, lhs, rhs, .. } => {
    let left = self.load_value(lhs, builder, variables)?;
    let right = self.load_value(rhs, builder, variables)?;
    let result = builder.ins().iadd(left, right);
    
    let var = Variable::new(*var_counter);
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
    
    let var = Variable::new(*var_counter);
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
    
    let var = Variable::new(*var_counter);
    *var_counter += 1;
    builder.declare_var(var, types::I64);
    builder.def_var(var, result);
    variables.insert(dest.clone(), var);
    
    Ok(())
}

IRInstruction::Div { dest, lhs, rhs, .. } => {
    let left = self.load_value(lhs, builder, variables)?;
    let right = self.load_value(rhs, builder, variables)?;
    // Signed division: sdiv
    let result = builder.ins().sdiv(left, right);
    
    let var = Variable::new(*var_counter);
    *var_counter += 1;
    builder.declare_var(var, types::I64);
    builder.def_var(var, result);
    variables.insert(dest.clone(), var);
    
    Ok(())
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/codegen/cranelift.rs
git commit -m "feat: implement arithmetic instructions (Add, Sub, Mul, Div) for Cranelift"
```

---

## Task 8: Implement Comparison Instructions

**Files:**
- Modify: `src/codegen/cranelift.rs`

**Step 1: Add comparison instruction handling**

Add to `translate_instruction`:

```rust
IRInstruction::Cmp { dest, op, lhs, rhs } => {
    let left = self.load_value(lhs, builder, variables)?;
    let right = self.load_value(rhs, builder, variables)?;
    
    // icmp returns an i8 (0 or 1)
    let cond = match op {
        CmpOp::Eq => IntCC::Equal,
        CmpOp::Ne => IntCC::NotEqual,
        CmpOp::Lt => IntCC::SignedLessThan,
        CmpOp::Le => IntCC::SignedLessThanOrEqual,
        CmpOp::Gt => IntCC::SignedGreaterThan,
        CmpOp::Ge => IntCC::SignedGreaterThanOrEqual,
    };
    
    let cmp_result = builder.ins().icmp(cond, left, right);
    // Extend i8 to i64 for consistency with our IR
    let result = builder.ins().uextend(types::I64, cmp_result);
    
    let var = Variable::new(*var_counter);
    *var_counter += 1;
    builder.declare_var(var, types::I64);
    builder.def_var(var, result);
    variables.insert(dest.clone(), var);
    
    Ok(())
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/codegen/cranelift.rs
git commit -m "feat: implement comparison instructions for Cranelift"
```

---

## Task 9: Implement Control Flow Instructions

**Files:**
- Modify: `src/codegen/cranelift.rs`

**Step 1: Add control flow instruction handling**

Add to `translate_instruction`:

```rust
IRInstruction::Label(name) => {
    // Get or create the block
    let block = block_map.entry(name.clone()).or_insert_with(|| {
        builder.create_block()
    });
    
    builder.switch_to_block(*block);
    
    // If all predecessors are known, seal the block
    // For now, we seal immediately (works for structured control flow)
    if !builder.is_block_sealed(*block) {
        builder.seal_block(*block);
    }
    
    Ok(())
}

IRInstruction::Jump(label) => {
    let block = block_map.entry(label.clone()).or_insert_with(|| {
        builder.create_block()
    });
    builder.ins().jump(*block, &[]);
    Ok(())
}

IRInstruction::CondBranch { condition, true_label, false_label } => {
    let cond_val = self.load_value(condition, builder, variables)?;
    
    // Compare with zero to get a boolean
    let zero = builder.ins().iconst(types::I64, 0);
    let cmp = builder.ins().icmp(IntCC::NotEqual, cond_val, zero);
    
    let true_block = block_map.entry(true_label.clone()).or_insert_with(|| {
        builder.create_block()
    });
    let false_block = block_map.entry(false_label.clone()).or_insert_with(|| {
        builder.create_block()
    });
    
    builder.ins().brif(cmp, *true_block, &[], *false_block, &[]);
    
    Ok(())
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/codegen/cranelift.rs
git commit -m "feat: implement control flow instructions for Cranelift"
```

---

## Task 10: Implement Function Calls

**Files:**
- Modify: `src/codegen/cranelift.rs`

**Step 1: Add function call handling**

Add to `translate_instruction`:

```rust
IRInstruction::Call { result, function, args } => {
    // Load all arguments
    let arg_values: Vec<Value> = args
        .iter()
        .map(|arg| self.load_value(arg, builder, variables))
        .collect::<Result<Vec<_>, _>>()?;
    
    // Create function signature for the call
    let mut sig = self.module.target_config().default_signature();
    for _ in args {
        sig.params.push(AbiParam::new(types::I64));
    }
    sig.returns.push(AbiParam::new(types::I64));
    
    // Declare the function (import if external)
    let func_id = self.module
        .declare_function(function, Linkage::Import, &sig)
        .map_err(|e| format!("Failed to declare function '{}': {}", function, e))?;
    
    // Create a function reference
    let func_ref = self.module.declare_func_in_func(func_id, builder.func);
    
    // Make the call
    let call_result = builder.ins().call(func_ref, &arg_values);
    
    // Store result if needed
    if let Some(dest) = result {
        let val = builder.inst_results(call_result)[0];
        
        let var = Variable::new(*var_counter);
        *var_counter += 1;
        builder.declare_var(var, types::I64);
        builder.def_var(var, val);
        variables.insert(dest.clone(), var);
    }
    
    Ok(())
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/codegen/cranelift.rs
git commit -m "feat: implement function calls for Cranelift"
```

---

## Task 11: Implement Module Generation

**Files:**
- Modify: `src/codegen/cranelift.rs`

**Step 1: Implement full CodeGenerator trait**

Update the `generate` method to actually compile functions:

```rust
impl CodeGenerator for CraneliftGenerator {
    fn generate(&mut self, module: &IRModule) -> Result<String, String> {
        // Compile all functions
        for func in &module.functions {
            self.compile_function(func)?;
        }
        
        // Return info message (actual output is via emit_object)
        Ok(format!(
            "; Generated {} functions with Cranelift",
            module.functions.len()
        ))
    }
}
```

**Step 2: Add method to compile and emit object file**

```rust
impl CraneliftGenerator {
    /// Compile a module and return the object file bytes
    pub fn compile_to_object(mut self, module: &IRModule) -> Result<Vec<u8>, String> {
        // Compile all functions
        for func in &module.functions {
            self.compile_function(func)?;
        }
        
        // Finalize and emit
        self.module
            .finish()
            .emit()
            .map_err(|e| format!("Failed to emit object file: {}", e))
    }
}
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/codegen/cranelift.rs
git commit -m "feat: implement full module generation for Cranelift"
```

---

## Task 12: Add Cranelift Target to Compiler

**Files:**
- Modify: `src/compiler.rs:1-148`

**Step 1: Add Cranelift target variant**

Update the Target enum:

```rust
pub enum Target {
    X86_64,
    ARM64,
    Cranelift,
}
```

**Step 2: Update default target**

Change default to Cranelift:

```rust
impl Compiler {
    pub fn new() -> Self {
        Compiler {
            output_name: "a.out".to_string(),
            keep_asm: false,
            target: Target::Cranelift, // Default to Cranelift for better code quality
        }
    }
    // ...
}
```

**Step 3: Update compile method for Cranelift**

Update the compile method's code generation section:

```rust
pub fn compile(&mut self, source: &str) -> Result<(), String> {
    // ... steps 1-4 unchanged ...
    
    // Step 5: Code Generation
    println!("\nStep 5: Code Generation");
    
    let obj_file = format!("{}.o", self.output_name);
    
    match self.target {
        Target::Cranelift => {
            use crate::codegen::CraneliftGenerator;
            
            let gen = CraneliftGenerator::new()
                .map_err(|e| format!("Failed to create Cranelift generator: {}", e))?;
            
            let obj_bytes = gen.compile_to_object(&module)
                .map_err(|e| format!("Code generation error: {}", e))?;
            
            // Write object file directly
            fs::write(&obj_file, &obj_bytes)
                .map_err(|e| format!("Failed to write object file: {}", e))?;
            
            println!("  ✓ Generated {} bytes of object code", obj_bytes.len());
        }
        
        Target::X86_64 => {
            let mut gen = X86_64Generator::new();
            let asm = gen.generate(&module)
                .map_err(|e| format!("Code generation error: {}", e))?;
            
            let asm_file = format!("{}.s", self.output_name);
            fs::write(&asm_file, &asm)
                .map_err(|e| format!("Failed to write assembly: {}", e))?;
            
            // Assemble
            let status = Command::new("as")
                .arg("-o")
                .arg(&obj_file)
                .arg(&asm_file)
                .status()
                .map_err(|e| format!("Failed to run assembler: {}", e))?;
            
            if !status.success() {
                return Err("Assembly failed".to_string());
            }
            
            if !self.keep_asm {
                fs::remove_file(&asm_file).ok();
            }
        }
        
        Target::ARM64 => {
            let mut gen = ARM64Generator::new();
            let asm = gen.generate(&module)
                .map_err(|e| format!("Code generation error: {}", e))?;
            
            let asm_file = format!("{}.s", self.output_name);
            fs::write(&asm_file, &asm)
                .map_err(|e| format!("Failed to write assembly: {}", e))?;
            
            let status = Command::new("as")
                .arg("-o")
                .arg(&obj_file)
                .arg(&asm_file)
                .status()
                .map_err(|e| format!("Failed to run assembler: {}", e))?;
            
            if !status.success() {
                return Err("Assembly failed".to_string());
            }
            
            if !self.keep_asm {
                fs::remove_file(&asm_file).ok();
            }
        }
    }
    
    println!("  ✓ Object file written to {}", obj_file);
    
    // Step 6: Link to executable
    let status = Command::new("clang")
        .arg("-o")
        .arg(&self.output_name)
        .arg(&obj_file)
        .status()
        .map_err(|e| format!("Failed to run linker: {}", e))?;
    
    if !status.success() {
        return Err("Linking failed".to_string());
    }
    println!("  ✓ Linked to executable {}", self.output_name);
    
    // Cleanup
    if !self.keep_asm {
        fs::remove_file(&obj_file).ok();
        println!("  ✓ Cleaned up temporary files");
    }
    
    println!("\n✅ Compilation successful!");
    println!("   Run with: ./{}", self.output_name);
    
    Ok(())
}
```

**Step 4: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add src/compiler.rs
git commit -m "feat: integrate Cranelift into compiler driver"
```

---

## Task 13: Add Integration Tests

**Files:**
- Create: `tests/cranelift_tests.rs`

**Step 1: Create integration test file**

```rust
//! Integration tests for Cranelift code generator

use ziv::codegen::CraneliftGenerator;
use ziv::ir::{IRFunction, IRInstruction, IRModule, IRType, IRValue};
use ziv::Compiler;

#[test]
fn test_cranelift_simple_return() {
    let mut module = IRModule::new();
    let mut func = IRFunction::new("main".to_string(), IRType::I64);
    
    func.add_instruction(IRInstruction::Ret {
        ty: IRType::I64,
        value: Some(IRValue::Const(42)),
    });
    
    module.add_function(func);
    
    let gen = CraneliftGenerator::new().expect("Failed to create generator");
    let obj_bytes = gen.compile_to_object(&module).expect("Failed to compile");
    
    assert!(!obj_bytes.is_empty(), "Object file should not be empty");
    // Check for ELF magic bytes
    assert_eq!(&obj_bytes[0..4], &[0x7f, 0x45, 0x4c, 0x46], "Should be valid ELF");
}

#[test]
fn test_cranelift_arithmetic() {
    let mut module = IRModule::new();
    let mut func = IRFunction::new("main".to_string(), IRType::I64);
    
    // let x = 10 + 20
    func.add_instruction(IRInstruction::Alloc {
        dest: "x".to_string(),
        ty: IRType::I64,
    });
    func.add_instruction(IRInstruction::Store {
        dest: "x".to_string(),
        ty: IRType::I64,
        value: IRValue::Const(10),
    });
    func.add_instruction(IRInstruction::Add {
        dest: "result".to_string(),
        ty: IRType::I64,
        lhs: IRValue::Var("x".to_string()),
        rhs: IRValue::Const(20),
    });
    func.add_instruction(IRInstruction::Ret {
        ty: IRType::I64,
        value: Some(IRValue::Var("result".to_string())),
    });
    
    module.add_function(func);
    
    let gen = CraneliftGenerator::new().expect("Failed to create generator");
    let obj_bytes = gen.compile_to_object(&module).expect("Failed to compile");
    
    assert!(!obj_bytes.is_empty());
}

#[test]
fn test_compile_simple_example() {
    let source = r#"
        let x = 42;
        let y = 10;
        let z = x + y;
    "#;
    
    let mut compiler = Compiler::new();
    compiler.output("test_simple");
    
    let result = compiler.compile(source);
    assert!(result.is_ok(), "Compilation should succeed: {:?}", result);
}

#[test]
fn test_compile_fibonacci() {
    let source = r#"
        let a = 0;
        let b = 1;
        let fib2 = a + b;
        let fib3 = b + fib2;
        let fib4 = fib2 + fib3;
    "#;
    
    let mut compiler = Compiler::new();
    compiler.output("test_fib");
    
    let result = compiler.compile(source);
    assert!(result.is_ok(), "Compilation should succeed: {:?}", result);
}
```

**Step 2: Run tests**

Run: `cargo test cranelift`
Expected: All tests PASS

**Step 3: Commit**

```bash
git add tests/cranelift_tests.rs
git commit -m "test: add integration tests for Cranelift codegen"
```

---

## Task 14: Update Documentation

**Files:**
- Modify: `CLAUDE.md:1-120`
- Modify: `README.md` (if exists)

**Step 1: Update CLAUDE.md with Cranelift info**

Add section after "## Architecture":

```markdown
### Code Generation Backends

LightLang supports multiple code generation backends:

**Cranelift (Default)**
- Production-quality code generation
- Built-in register allocation
- Multi-architecture support (x86-64, ARM64, etc.)
- Direct object file emission (no external assembler)

**Legacy (x86-64/ARM64 assembly)**
- Custom assembly generators
- Requires external `as` assembler
- Limited optimization

The Cranelift backend is recommended for production use.
```

**Step 2: Commit**

```bash
git add CLAUDE.md README.md
git commit -m "docs: update documentation for Cranelift backend"
```

---

## Task 15: Final Verification and Cleanup

**Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests PASS

**Step 2: Build release binary**

Run: `cargo build --release`
Expected: Builds without errors

**Step 3: Test with example files**

Run:
```bash
./target/release/llc examples/simple.ll -o simple_test
./simple_test
echo $?  # Should show 0
```

Expected: Program compiles and runs successfully

**Step 4: Test with fib_simple.ll**

Run:
```bash
./target/release/llc examples/fib_simple.ll -o fib_test
./fib_test
echo $?
```

Expected: Program compiles and runs successfully

**Step 5: Final commit**

```bash
git add -A
git commit -m "feat: complete Cranelift code generator migration"
```

---

## Summary

| Task | Description | Files Modified |
|------|-------------|----------------|
| 1 | Add Cranelift dependencies | Cargo.toml |
| 2 | Create generator module | src/codegen/cranelift.rs, mod.rs |
| 3 | Type translation | cranelift.rs |
| 4 | Function context | cranelift.rs |
| 5 | Function declaration | cranelift.rs |
| 6 | Memory instructions | cranelift.rs |
| 7 | Arithmetic instructions | cranelift.rs |
| 8 | Comparison instructions | cranelift.rs |
| 9 | Control flow | cranelift.rs |
| 10 | Function calls | cranelift.rs |
| 11 | Module generation | cranelift.rs |
| 12 | Compiler integration | compiler.rs |
| 13 | Integration tests | tests/cranelift_tests.rs |
| 14 | Documentation | CLAUDE.md, README.md |
| 15 | Final verification | - |

**Benefits of Cranelift:**
- Production-quality code generation
- Built-in register allocation (no manual stack management)
- Multi-architecture support
- Direct object file emission (no external assembler)
- Active development and security updates
