use lightlang::codegen::ARM64Generator;
use lightlang::ir::IRModule;

#[test]
fn test_arm64_generator_creation() {
    let generator = ARM64Generator::new();
    let module = IRModule::new();
    let result = generator.generate(&module);
    assert!(result.is_ok());
}
