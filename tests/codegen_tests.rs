use ziv::codegen::ARM64Generator;
use ziv::ir::IRModule;
use ziv::CodeGenerator;

#[test]
fn test_arm64_generator_creation() {
    let mut generator = ARM64Generator::new();
    let module = IRModule::new();
    let result = generator.generate(&module);
    assert!(result.is_ok());
}
