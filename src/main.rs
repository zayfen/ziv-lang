//! LightLang compiler CLI

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <source.ll>", args[0]);
        eprintln!("  Options:");
        eprintln!("    -o <output>    Output executable name");
        eprintln!("    --keep-asm     Keep assembly files");
        process::exit(1);
    }
    
    let mut source_file = None;
    let mut output_name = "a.out".to_string();
    let mut keep_asm = false;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                if i < args.len() {
                    output_name = args[i].clone();
                } else {
                    eprintln!("Error: -o requires an argument");
                    process::exit(1);
                }
            },
            "--keep-asm" => {
                keep_asm = true;
            },
            file => {
                source_file = Some(file.to_string());
            }
        }
        i += 1;
    }
    
    let source_file = match source_file {
        Some(f) => f,
        None => {
            eprintln!("Error: No source file specified");
            process::exit(1);
        }
    };
    
    // Read source file
    let source = fs::read_to_string(&source_file)
        .unwrap_or_else(|e| {
            eprintln!("Error reading file: {}", e);
            process::exit(1);
        });
    
    println!("Compiling: {}", source_file);
    println!();
    
    // Compile
    let mut compiler = lightlang::Compiler::new()
        .output(&output_name)
        .keep_asm(keep_asm);
    
    if let Err(e) = compiler.compile(&source) {
        eprintln!("Compilation error: {}", e);
        process::exit(1);
    }
}
