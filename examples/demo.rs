use ziv::*;

fn main() {
    println!("=== LightLang Compiler Demo ===\n");
    
    // Source code
    let code = r#"
        let x = 42;
        let y = 10;
        let z = x + y;
    "#;
    
    println!("Source code:");
    println!("{}\n", code);
    
    // Step 1: Lexing
    println!("Step 1: Lexing");
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().unwrap();
    println!("  Tokens: {} generated\n", tokens.len());
    
    // Step 2: Parsing
    println!("Step 2: Parsing");
    let mut parser = Parser::new(code);
    let program = parser.parse().unwrap();
    println!("  AST: {} statements\n", program.statements.len());
    
    // Step 3: Semantic Analysis
    println!("Step 3: Semantic Analysis");
    let analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    println!("  Status: {:?}\n", result);
    
    // Step 4: IR Generation
    println!("Step 4: IR Generation");
    let builder = IRBuilder::new();
    let module = builder.build(&program);
    println!("  Functions: {}", module.functions.len());
    println!("  IR:\n{}\n", module);
    
    // Step 5: Code Generation (LLVM IR)
    println!("Step 5: LLVM IR Generation");
    let gen = LLVMTextGenerator::new();
    let output = gen.generate(&module).unwrap();
    println!("{}\n", output);
    
    // Step 6: Code Generation (x86_64)
    println!("Step 6: x86_64 Assembly Generation");
    let gen = X86_64Generator::new();
    let output = gen.generate(&module).unwrap();
    println!("{}\n", output);
    
    println!("=== Compilation Complete! ===");
}
