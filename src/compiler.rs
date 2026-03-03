//! LightLang compiler driver

use std::fs;
use std::process::Command;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use crate::ir::IRBuilder;
use crate::codegen::X86_64Generator;
use crate::codegen::CodeGenerator;

pub struct Compiler {
    output_name: String,
    keep_asm: bool,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            output_name: "a.out".to_string(),
            keep_asm: false,
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
    
    pub fn compile(&mut self, source: &str) -> Result<(), String> {
        // Step 1: Lexing
        println!("Step 1: Lexing");
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()
            .map_err(|e| format!("Lexer error: {}", e))?;
        println!("  ✓ Generated {} tokens", tokens.len());
        
        // Step 2: Parsing
        println!("\nStep 2: Parsing");
        let mut parser = Parser::new(source);
        let program = parser.parse()
            .map_err(|e| format!("Parser error: {}", e))?;
        println!("  ✓ Parsed {} statements", program.statements.len());
        
        // Step 3: Semantic Analysis
        println!("\nStep 3: Semantic Analysis");
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&program)
            .map_err(|e| format!("Semantic error: {}", e))?;
        println!("  ✓ Semantic analysis passed");
        
        // Step 4: IR Generation
        println!("\nStep 4: IR Generation");
        let builder = IRBuilder::new();
        let module = builder.build(&program);
        println!("  ✓ Generated IR with {} functions", module.functions.len());
        
        // Step 5: Code Generation
        println!("\nStep 5: Code Generation");
        let mut gen = X86_64Generator::new();
        let asm = gen.generate(&module)
            .map_err(|e| format!("Code generation error: {}", e))?;
        println!("  ✓ Generated {} bytes of assembly", asm.len());
        
        // Step 6: Write assembly file
        let asm_file = format!("{}.s", self.output_name);
        fs::write(&asm_file, asm)
            .map_err(|e| format!("Failed to write assembly: {}", e))?;
        println!("  ✓ Wrote assembly to {}", asm_file);
        
        // Step 7: Assemble to object file
        let obj_file = format!("{}.o", self.output_name);
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
        
        // Step 8: Link to executable
        let status = Command::new("ld")
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
            fs::remove_file(&asm_file).ok();
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
