//! LLVM IR text generator

use crate::codegen::CodeGenerator;
use crate::ir::IRModule;

pub struct LLVMTextGenerator;

impl LLVMTextGenerator {
    pub fn new() -> Self {
        LLVMTextGenerator
    }
}

impl CodeGenerator for LLVMTextGenerator {
    fn generate(&mut self, module: &IRModule) -> Result<String, String> {
        let mut output = String::new();

        output.push_str("; LightLang LLVM IR\n\n");
        output.push_str("; External declarations\n");
        output.push_str("declare i32 @printf(i8*, ...)\n");
        output.push_str("declare i32 @scanf(i8*, ...)\n\n");

        for func in &module.functions {
            output.push_str(&format!("{}", func));
            output.push_str("\n");
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llvm_generator() {
        let mut gen = LLVMTextGenerator::new();
        let module = IRModule::new();
        let result = gen.generate(&module);
        assert!(result.is_ok());
    }
}
