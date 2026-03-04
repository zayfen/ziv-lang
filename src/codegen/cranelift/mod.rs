mod crate::codegen::CodeGenerator;
use crate::ir::{CmpOp, IRFunction, IRInstruction, IRModule, IRType, IRValue};
use cranelift_codegen::ir::types;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_module::default_libcall_names;
use cranelift_object::{ObjectBuilder, ObjectModule};

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
        
        let builder = ObjectBuilder::new(
            isa,
            "lightlang",
            default_libcall_names(),
        )
        .map_err(|e| format!("Failed to create object builder: {}", e))?;
        
        Ok(CraneliftGenerator { module: ObjectModule::new(builder) })
    }
}

impl CodeGenerator for CraneliftGenerator {
    fn generate(&mut self, _module: &IRModule) -> Result<String, String> {
        Ok("; Cranelift generates object files, not assembly".to_string())
    }
}

impl CraneliftGenerator {
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
