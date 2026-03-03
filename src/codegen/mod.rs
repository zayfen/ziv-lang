//! Code generation for LightLang

pub mod x86_64;
pub mod llvm_text;

use crate::ir::IRModule;

pub type CodeGenResult<T> = Result<T, String>;

/// Code generator trait
pub trait CodeGenerator {
    fn generate(&mut self, module: &IRModule) -> CodeGenResult<String>;
}

// Re-export generators
pub use x86_64::X86_64Generator;
pub use llvm_text::LLVMTextGenerator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_module() {
        assert!(true);
    }
}
