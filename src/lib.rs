//! LightLang - A modern systems programming language

pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod ir;
pub mod codegen;
pub mod compiler;

pub use lexer::{Lexer, Token};
pub use parser::Parser;
pub use semantic::{SemanticAnalyzer, Type, Symbol, SymbolKind};
pub use ir::{IRModule, IRBuilder};
pub use codegen::{CodeGenerator, X86_64Generator, LLVMTextGenerator};
pub use compiler::Compiler;
