//! LightLang compiler driver

use crate::codegen::ARM64Generator;
use crate::codegen::CodeGenerator;
use crate::codegen::CraneliftGenerator;
use crate::codegen::X86_64Generator;
use crate::ir::IRBuilder;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use std::fs;
use std::process::Command;

pub enum Target {
    X86_64,
    ARM64,
    Cranelift,
}

pub struct Compiler {
    output_name: String,
    keep_asm: bool,
    target: Target,
    assembler_cmd: String,
    linker_cmd: String,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            output_name: "a.out".to_string(),
            keep_asm: false,
            target: Target::Cranelift, // Default to Cranelift for better code quality
            assembler_cmd: "as".to_string(),
            linker_cmd: "clang".to_string(),
        }
    }

    pub fn output(mut self, name: &str) -> Self {
        self.output_name = name.to_string();
        self
    }

    pub fn keep_asm(mut self, keep: bool) -> Self {
        self.keep_asm = keep;
        self
    }

    pub fn target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    pub fn assembler(mut self, cmd: &str) -> Self {
        self.assembler_cmd = cmd.to_string();
        self
    }

    pub fn linker(mut self, cmd: &str) -> Self {
        self.linker_cmd = cmd.to_string();
        self
    }

    pub fn compile(&mut self, source: &str) -> Result<(), String> {
        // Step 1: Lexing
        println!("Step 1: Lexing");
        let mut lexer = Lexer::new(source);
        let tokens = lexer
            .tokenize()
            .map_err(|e| format!("Lexer error: {}", e))?;
        println!("  ✓ Generated {} tokens", tokens.len());

        // Step 2: Parsing
        println!("\nStep 2: Parsing");
        let mut parser = Parser::new(source);
        let program = parser.parse().map_err(|e| format!("Parser error: {}", e))?;
        println!("  ✓ Parsed {} statements", program.statements.len());

        // Step 3: Semantic Analysis
        println!("\nStep 3: Semantic Analysis");
        let mut analyzer = SemanticAnalyzer::new();
        analyzer
            .analyze(&program)
            .map_err(|e| format!("Semantic error: {}", e))?;
        println!("  ✓ Semantic analysis passed");

        // Step 4: IR Generation
        println!("\nStep 4: IR Generation");
        let builder = IRBuilder::new();
        let module = builder.build(&program);
        println!("  ✓ Generated IR with {} functions", module.functions.len());

        // Step 5: Code Generation
        println!("\nStep 5: Code Generation");

        let obj_file = format!("{}.o", self.output_name);

        match self.target {
            Target::Cranelift => {
                let gen = CraneliftGenerator::new()?;

                let obj_bytes = gen.compile_to_object(&module)?;

                fs::write(&obj_file, &obj_bytes)
                    .map_err(|e| format!("Failed to write object file: {}", e))?;

                println!("  ✓ Generated {} bytes of object code", obj_bytes.len());

                // Detect architecture and generate appropriate start helper
                // On macOS, the entry point is _main, so we create a wrapper
                // that calls our __user_main and returns to the C runtime.
                #[cfg(target_arch = "aarch64")]
                let start_asm = r#"
.text
.globl _main
_main:
    stp x29, x30, [sp, #-16]!
    mov x29, sp
    bl __user_main
    ldp x29, x30, [sp], #16
    ret
"#;
                #[cfg(target_arch = "x86_64")]
                let start_asm = r#"
.text
.globl _main
_main:
    pushq %rbp
    movq %rsp, %rbp
    call __user_main
    popq %rbp
    ret
"#;

                let start_asm_file = format!("{}_start.s", self.output_name);
                fs::write(&start_asm_file, start_asm)
                    .map_err(|e| format!("Failed to write start assembly: {}", e))?;

                let start_obj_file = format!("{}_start.o", self.output_name);
                let mut assembler = Command::new(&self.assembler_cmd);
                #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
                assembler.arg("-arch").arg("arm64");
                #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
                assembler.arg("-arch").arg("x86_64");
                let status = assembler
                    .arg("-o")
                    .arg(&start_obj_file)
                    .arg(&start_asm_file)
                    .status()
                    .map_err(|e| format!("Failed to run assembler: {}", e))?;

                if !status.success() {
                    return Err("Assembly of start helper failed".to_string());
                }

                // Build stdlib runtime object that provides callable built-ins.
                let runtime_c_file = format!("{}_stdlib_runtime.c", self.output_name);
                let runtime_obj_file = format!("{}_stdlib_runtime.o", self.output_name);
                let runtime_c = r#"
#include <stdint.h>
#include <stdio.h>

int64_t ziv_print_i64(int64_t value) {
    printf("%lld", (long long)value);
    return 0;
}

int64_t ziv_println_i64(int64_t value) {
    printf("%lld\n", (long long)value);
    return 0;
}

int64_t ziv_print_str(const char* value) {
    if (value != NULL) {
        fputs(value, stdout);
    }
    return 0;
}

int64_t ziv_println_str(const char* value) {
    if (value != NULL) {
        fputs(value, stdout);
    }
    fputc('\n', stdout);
    return 0;
}
"#;
                fs::write(&runtime_c_file, runtime_c)
                    .map_err(|e| format!("Failed to write stdlib runtime source: {}", e))?;

                let status = Command::new(&self.linker_cmd)
                    .arg("-c")
                    .arg(&runtime_c_file)
                    .arg("-o")
                    .arg(&runtime_obj_file)
                    .status()
                    .map_err(|e| format!("Failed to run linker: {}", e))?;
                if !status.success() {
                    return Err("Compilation of stdlib runtime failed".to_string());
                }

                // Link with both object files
                let status = Command::new(&self.linker_cmd)
                    .arg("-o")
                    .arg(&self.output_name)
                    .arg(&obj_file)
                    .arg(&start_obj_file)
                    .arg(&runtime_obj_file)
                    .status()
                    .map_err(|e| format!("Failed to run linker: {}", e))?;

                if !status.success() {
                    return Err("Linking failed".to_string());
                }
                println!("  ✓ Linked to executable {}", self.output_name);

                // Cleanup
                if !self.keep_asm {
                    fs::remove_file(&start_asm_file).ok();
                    fs::remove_file(&start_obj_file).ok();
                    fs::remove_file(&runtime_c_file).ok();
                    fs::remove_file(&runtime_obj_file).ok();
                    fs::remove_file(&obj_file).ok();
                    println!("  ✓ Cleaned up temporary files");
                } else {
                    fs::remove_file(&runtime_c_file).ok();
                }

                println!("\n✅ Compilation successful!");
                println!("   Run with: ./{}", self.output_name);

                return Ok(());
            }

            Target::X86_64 => {
                let mut gen = X86_64Generator::new();
                let asm = gen.generate(&module)?;

                let asm_file = format!("{}.s", self.output_name);
                fs::write(&asm_file, &asm)
                    .map_err(|e| format!("Failed to write assembly: {}", e))?;

                println!("  ✓ Generated {} bytes of assembly", asm.len());

                let mut assembler = Command::new(&self.assembler_cmd);
                #[cfg(target_os = "macos")]
                assembler.arg("-arch").arg("x86_64");
                let status = assembler
                    .arg("-o")
                    .arg(&obj_file)
                    .arg(&asm_file)
                    .status()
                    .map_err(|e| format!("Failed to run assembler: {}", e))?;

                if !status.success() {
                    return Err("Assembly failed".to_string());
                }
                println!("  ✓ Assembled to {}", obj_file);

                if !self.keep_asm {
                    fs::remove_file(&asm_file).ok();
                }
            }

            Target::ARM64 => {
                let mut gen = ARM64Generator::new();
                let asm = gen.generate(&module)?;

                let asm_file = format!("{}.s", self.output_name);
                fs::write(&asm_file, &asm)
                    .map_err(|e| format!("Failed to write assembly: {}", e))?;

                println!("  ✓ Generated {} bytes of assembly", asm.len());

                let mut assembler = Command::new(&self.assembler_cmd);
                #[cfg(target_os = "macos")]
                assembler.arg("-arch").arg("arm64");
                let status = assembler
                    .arg("-o")
                    .arg(&obj_file)
                    .arg(&asm_file)
                    .status()
                    .map_err(|e| format!("Failed to run assembler: {}", e))?;

                if !status.success() {
                    return Err("Assembly failed".to_string());
                }
                println!("  ✓ Assembled to {}", obj_file);

                if !self.keep_asm {
                    fs::remove_file(&asm_file).ok();
                }
            }
        }

        println!("  ✓ Object file written to {}", obj_file);

        // Step 6: Link to executable
        let mut linker = Command::new(&self.linker_cmd);
        #[cfg(target_os = "macos")]
        if matches!(self.target, Target::X86_64) {
            linker.arg("-arch").arg("x86_64");
        }
        let status = linker
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler::new();
        assert_eq!(compiler.output_name, "a.out");
    }

    #[test]
    fn test_compiler_builder_methods() {
        let compiler = Compiler::new()
            .output("out.bin")
            .keep_asm(true)
            .target(Target::ARM64)
            .assembler("my-as")
            .linker("my-linker");
        assert_eq!(compiler.output_name, "out.bin");
        assert!(compiler.keep_asm);
        assert_eq!(
            std::mem::discriminant(&compiler.target),
            std::mem::discriminant(&Target::ARM64)
        );
        assert_eq!(compiler.assembler_cmd, "my-as");
        assert_eq!(compiler.linker_cmd, "my-linker");
    }

    #[test]
    fn test_compile_lexer_error() {
        let huge = format!("let x = {};", "9".repeat(200));
        let mut compiler = Compiler::new().output("lexer_err_bin");
        let err = compiler.compile(&huge).unwrap_err();
        assert!(err.contains("Lexer error"));
        fs::remove_file("lexer_err_bin").ok();
    }

    #[test]
    fn test_compile_parser_and_semantic_errors() {
        let mut parser_err = Compiler::new().output("parser_err_bin");
        let err = parser_err.compile("/").unwrap_err();
        assert!(err.contains("Parser error"));

        let mut semantic_err = Compiler::new().output("semantic_err_bin");
        let err = semantic_err.compile("let y = x;").unwrap_err();
        assert!(err.contains("Semantic error"));
    }

    #[test]
    fn test_compile_cranelift_success_and_cleanup() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_ok");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str);
        compiler.compile("let x = 1; let y = x + 2;").unwrap();

        assert!(output.exists());
        assert!(!dir.path().join("cranelift_ok.o").exists());
        assert!(!dir.path().join("cranelift_ok_start.s").exists());
        assert!(!dir.path().join("cranelift_ok_start.o").exists());
        assert!(!dir.path().join("cranelift_ok_stdlib_runtime.c").exists());
        assert!(!dir.path().join("cranelift_ok_stdlib_runtime.o").exists());
    }

    #[test]
    fn test_compile_cranelift_keep_asm() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_keep");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str).keep_asm(true);
        compiler.compile("let x = 1;").unwrap();

        assert!(output.exists());
        assert!(dir.path().join("cranelift_keep.o").exists());
        assert!(dir.path().join("cranelift_keep_start.s").exists());
        assert!(dir.path().join("cranelift_keep_start.o").exists());
        assert!(dir.path().join("cranelift_keep_stdlib_runtime.o").exists());
        assert!(!dir.path().join("cranelift_keep_stdlib_runtime.c").exists());
    }

    #[test]
    fn test_compile_cranelift_write_object_failure() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing").join("out");
        let output_str = missing.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str);
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Failed to write object file"));
    }

    #[test]
    fn test_compile_cranelift_link_failure_with_directory_output() {
        let dir = tempdir().unwrap();
        let output_str = dir.path().to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str);
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Linking failed"));

        let obj = format!("{}.o", output_str);
        let start_s = format!("{}_start.s", output_str);
        let start_o = format!("{}_start.o", output_str);
        let runtime_c = format!("{}_stdlib_runtime.c", output_str);
        let runtime_o = format!("{}_stdlib_runtime.o", output_str);
        fs::remove_file(obj).ok();
        fs::remove_file(start_s).ok();
        fs::remove_file(start_o).ok();
        fs::remove_file(runtime_c).ok();
        fs::remove_file(runtime_o).ok();
    }

    #[test]
    fn test_compile_cranelift_start_helper_assembly_failure() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_start_fail");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str).assembler("false");
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Assembly of start helper failed"));
    }

    #[test]
    fn test_compile_cranelift_start_helper_spawn_error() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_start_spawn_fail");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .assembler("__ziv_missing_assembler__");
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Failed to run assembler"));
    }

    #[test]
    fn test_compile_cranelift_linker_spawn_error() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_link_spawn_fail");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .linker("__ziv_missing_linker__");
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Failed to run linker"));
    }

    #[test]
    fn test_compile_arm64_success() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("arm_ok");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .target(Target::ARM64);
        compiler
            .compile("function main() { return 0; }")
            .unwrap();
        assert!(output.exists());
    }

    #[test]
    fn test_compile_arm64_success_keep_asm() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("arm_keep");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .target(Target::ARM64)
            .keep_asm(true);
        compiler
            .compile("function main() { return 0; }")
            .unwrap();

        assert!(output.exists());
        assert!(dir.path().join("arm_keep.o").exists());
        assert!(dir.path().join("arm_keep.s").exists());
    }

    #[test]
    fn test_compile_arm64_and_x86_assembly_failures() {
        let dir = tempdir().unwrap();
        let arm_out = dir.path().join("arm_fail");
        let arm_out_str = arm_out.to_string_lossy().to_string();
        let mut arm_compiler = Compiler::new()
            .output(&arm_out_str)
            .target(Target::ARM64);
        let arm_err = arm_compiler.compile("let x = 2 * 3;").unwrap_err();
        assert!(arm_err.contains("Assembly failed") | arm_err.contains("Failed to run assembler"));

        let x86_out = dir.path().join("x86_fail");
        let x86_out_str = x86_out.to_string_lossy().to_string();
        let mut x86_compiler = Compiler::new()
            .output(&x86_out_str)
            .target(Target::X86_64);
        let x86_err = x86_compiler
            .compile("while (1) { let y = 1; }")
            .unwrap_err();
        assert!(x86_err.contains("Assembly failed") | x86_err.contains("Failed to run assembler"));
    }

    #[test]
    fn test_compile_x86_assembly_success_then_link_failure_and_cleanup() {
        let dir = tempdir().unwrap();
        let output_str = dir.path().to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .target(Target::X86_64);
        let err = compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(err.contains("Linking failed"));

        let asm_file = format!("{}.s", output_str);
        assert!(!std::path::Path::new(&asm_file).exists());
    }

    #[test]
    fn test_compile_x86_assembly_success_then_link_failure_keep_asm() {
        let dir = tempdir().unwrap();
        let output_str = dir.path().to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .target(Target::X86_64)
            .keep_asm(true);
        let err = compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(err.contains("Linking failed"));

        let asm_file = format!("{}.s", output_str);
        assert!(std::path::Path::new(&asm_file).exists());
    }

    #[test]
    fn test_compile_x86_and_arm64_assembler_spawn_errors() {
        let dir = tempdir().unwrap();

        let x86_out = dir.path().join("x86_spawn_fail");
        let x86_out_str = x86_out.to_string_lossy().to_string();
        let mut x86_compiler = Compiler::new()
            .output(&x86_out_str)
            .target(Target::X86_64)
            .assembler("__ziv_missing_assembler__");
        let x86_err = x86_compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(x86_err.contains("Failed to run assembler"));

        let arm_out = dir.path().join("arm_spawn_fail");
        let arm_out_str = arm_out.to_string_lossy().to_string();
        let mut arm_compiler = Compiler::new()
            .output(&arm_out_str)
            .target(Target::ARM64)
            .assembler("__ziv_missing_assembler__");
        let arm_err = arm_compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(arm_err.contains("Failed to run assembler"));
    }

    #[test]
    fn test_compile_arm64_linker_spawn_error() {
        let dir = tempdir().unwrap();
        let out = dir.path().join("arm_link_spawn_fail");
        let out_str = out.to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&out_str)
            .target(Target::ARM64)
            .linker("__ziv_missing_linker__");
        let err = compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(err.contains("Failed to run linker"));
    }
}
