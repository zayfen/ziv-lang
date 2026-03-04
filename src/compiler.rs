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
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            output_name: "a.out".to_string(),
            keep_asm: false,
            target: Target::Cranelift, // Default to Cranelift for better code quality
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
                let gen = CraneliftGenerator::new()
                    .map_err(|e| format!("Failed to create Cranelift generator: {}", e))?;

                let obj_bytes = gen
                    .compile_to_object(&module)
                    .map_err(|e| format!("Code generation error: {}", e))?;

                fs::write(&obj_file, &obj_bytes)
                    .map_err(|e| format!("Failed to write object file: {}", e))?;

                println!("  ✓ Generated {} bytes of object code", obj_bytes.len());

                // Detect architecture and generate appropriate start helper
                // On macOS, the entry point is _main, so we create a wrapper
                // that calls our __user_main and exits
                #[cfg(target_arch = "aarch64")]
                let start_asm = r#"
.text
.globl _main
_main:
    stp x29, x30, [sp, #-16]!
    bl __user_main
    mov x0, x0
    mov x16, #1        // exit syscall
    svc #0
"#;
                #[cfg(target_arch = "x86_64")]
                let start_asm = r#"
.text
.globl _main
_main:
    call __user_main
    movq %rax, %rdi
    movq $0x2000001, %rax  // exit syscall on macOS
    syscall
"#;

                let start_asm_file = format!("{}_start.s", self.output_name);
                fs::write(&start_asm_file, start_asm)
                    .map_err(|e| format!("Failed to write start assembly: {}", e))?;

                let start_obj_file = format!("{}_start.o", self.output_name);
                let status = Command::new("as")
                    .arg("-o")
                    .arg(&start_obj_file)
                    .arg(&start_asm_file)
                    .status()
                    .map_err(|e| format!("Failed to run assembler: {}", e))?;

                if !status.success() {
                    return Err("Assembly of start helper failed".to_string());
                }

                // Link with both object files
                let status = Command::new("clang")
                    .arg("-o")
                    .arg(&self.output_name)
                    .arg(&obj_file)
                    .arg(&start_obj_file)
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
                    fs::remove_file(&obj_file).ok();
                    println!("  ✓ Cleaned up temporary files");
                }

                println!("\n✅ Compilation successful!");
                println!("   Run with: ./{}", self.output_name);

                return Ok(());
            }

            Target::X86_64 => {
                let mut gen = X86_64Generator::new();
                let asm = gen
                    .generate(&module)
                    .map_err(|e| format!("Code generation error: {}", e))?;

                let asm_file = format!("{}.s", self.output_name);
                fs::write(&asm_file, &asm)
                    .map_err(|e| format!("Failed to write assembly: {}", e))?;

                println!("  ✓ Generated {} bytes of assembly", asm.len());

                let status = Command::new("as")
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
                let asm = gen
                    .generate(&module)
                    .map_err(|e| format!("Code generation error: {}", e))?;

                let asm_file = format!("{}.s", self.output_name);
                fs::write(&asm_file, &asm)
                    .map_err(|e| format!("Failed to write assembly: {}", e))?;

                println!("  ✓ Generated {} bytes of assembly", asm.len());

                let status = Command::new("as")
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler::new();
        assert_eq!(compiler.output_name, "a.out");
    }
}
