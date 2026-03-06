# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LightLang is a modern systems programming language implemented in Rust that compiles to native ELF executables. The design philosophy combines JavaScript's elegant syntax with C-level performance and modern memory safety features.

## Build Commands

```bash
# Build the compiler
cargo build

# Build in release mode (optimized)
cargo build --release

# Run tests
cargo test

# Run specific test module
cargo test lexer
cargo test parser

# Install the compiler locally
cargo install --path .

# Compile a LightLang source file
llc source.ll -o program
llc source.ll -o program --keep-asm  # Keep intermediate assembly files
```

## Development Workflow

### Running the Compiler

After building, the compiler binary is `llc`:

```bash
# Compile a simple program
./target/debug/llc examples/simple.ll -o test_program

# Run the generated executable
./test_program
echo $?  # Check exit code
```

### Testing Strategy

The project uses Rust's built-in test framework with tests organized in the `tests/` directory:
- `lexer_tests.rs` - Lexical analysis tests
- `parser_tests.rs` - Parsing tests
- `integration_tests.rs` - End-to-end compilation tests

## Architecture

LightLang follows a traditional multi-pass compiler architecture:

### Compilation Pipeline

```
Source Code (.ll)
    ↓
[1] Lexer (logos) → Token Stream
    ↓
[2] Parser (LALRPOP) → Abstract Syntax Tree (AST)
    ↓
[3] Semantic Analyzer → Type-annotated AST + Symbol Table
    ↓
[4] IR Builder → Custom Intermediate Representation
    ↓
[5] Code Generator (x86_64) → Assembly (.s)
    ↓
[6] System Assembler (as) → Object File (.o)
    ↓
[7] System Linker (ld) → ELF Executable
```

### Module Structure

**src/lexer/** - Lexical Analysis
- Uses `logos` crate for lexer generation
- Converts source text into tokens
- Handles identifiers, literals, keywords, operators

**src/parser/** - Syntax Analysis
- Uses `lalrpop` parser generator (grammar files in `src/parser/`)
- Produces Abstract Syntax Tree (AST)
- Core AST types defined in `parser/ast.rs`

**src/semantic/** - Semantic Analysis
- Type checking and type inference
- Symbol table management (`symbols.rs`)
- Type system definitions (`types.rs`)
- Scope management and name resolution

**src/ir/** - Intermediate Representation
- Custom IR for optimization and code generation
- Three-address code representation
- SSA-like variable naming (`t0`, `t1`, etc.)
- Instructions: `Assign`, `BinaryOp`, `Return`, etc.

**src/codegen/** - Code Generation
- `x86_64.rs`: Generates x86-64 assembly from IR
- `llvm_text.rs`: Alternative LLVM IR text format generator
- Stack-based variable allocation
- System call integration (Linux x86-64 ABI)

**src/compiler.rs** - Compiler Driver
- Orchestrates all compilation phases
- Manages temporary files (`.s`, `.o`)
- Invokes external tools (`as`, `ld`)

### Key Design Decisions

**Memory Management**: Region-based inference instead of garbage collection - variables are automatically freed when leaving scope.

**Type System**: Static typing with type inference. Supports primitive types (i32, i64, f32, f64, bool, char, string), arrays, and user-defined types.

**Output Target**: Native ELF executables for x86-64 Linux. No runtime dependencies.

**Entry Point**: Programs must have a `main` function. The compiler generates `_start` that calls `main`.

## Language Syntax

LightLang uses JavaScript-inspired syntax with type annotations:

```ziv
// Variable declarations
let x = 42;
let y: i64 = 100;
const PI = 3.14159;

// Functions
function add(a: i32, b: i32): i32 {
    return a + b;
}

// Arrow functions
const multiply = (a: i32, b: i32): i32 => a * b;

// Control flow
if (x > 0) {
    // ...
} else {
    // ...
}

// Loops
for (let i = 0; i < 10; i++) {
    // ...
}

while (condition) {
    // ...
}
```

## Dependencies

**Core Dependencies**:
- `logos` - Lexer generator
- `lalrpop` - Parser generator
- `thiserror`, `anyhow`, `miette` - Error handling
- `goblin`, `scroll` - ELF manipulation

**Build Dependencies**:
- `lalrpop` - Compiles `.lalrpop` grammar files during build

**Dev Dependencies**:
- `insta` - Snapshot testing
- `tempfile` - Temporary file management in tests

## Working with the Codebase

### Adding New Language Features

1. **Lexer**: Add new token types in `src/lexer/lib.rs`
2. **Parser**: Update grammar in `src/parser/*.lalrpop` files
3. **AST**: Extend AST node types in `src/parser/ast.rs`
4. **Semantic**: Add type checking logic in `src/semantic/`
5. **IR**: Extend IR instructions in `src/ir/instructions.rs`
6. **Codegen**: Implement code generation in `src/codegen/x86_64.rs`

### Debugging Compilation

The compiler outputs progress information at each phase:
```
Step 1: Lexing
  ✓ Generated X tokens

Step 2: Parsing
  ✓ Parsed X statements
...
```

Use `--keep-asm` flag to inspect generated assembly:
```bash
llc source.ll -o program --keep-asm
cat program.s  # View generated assembly
```

### Error Handling

The compiler uses `miette` for rich error diagnostics with source code spans and suggestions. Errors are propagated using `Result<T, String>` through the pipeline.

## Testing Examples

The `examples/` directory contains test programs:
- `simple.ll` - Basic variable declarations and arithmetic
- `fib_simple.ll` - Fibonacci calculation (non-recursive)
- `return52.ll` - Simple return value test
- `test_basic.ll` - Basic functionality test

Run an example:
```bash
./target/debug/llc examples/simple.ll -o simple_test
./simple_test
```
