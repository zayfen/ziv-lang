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
    use crate::ir::IRModule;

    #[test]
    fn test_codegen_module() {
        let _ = ARM64Generator::new();
        let _ = X86_64Generator::new();
        let _ = LLVMTextGenerator::new();
        let mut gen = CraneliftGenerator::default();
        let module = IRModule::new();
        assert!(gen.generate(&module).is_ok());
    }
}
