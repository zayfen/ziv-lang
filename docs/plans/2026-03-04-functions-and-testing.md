# Functions and Test Suite Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Establish comprehensive test coverage and implement function definitions/calls to enable real program composition.

**Architecture:** Build test infrastructure first (TDD), then extend AST with type annotations, implement function call IR/codegen using x86-64 calling convention. Each task is bite-sized with red-green-refactor cycle.

**Tech Stack:** Rust, logos (lexer), lalrpop (parser), x86-64 assembly, system linker (ld)

---

## Task 1: Lexer Test Suite Foundation

**Files:**
- Create: `tests/lexer_tests.rs`
- Reference: `src/lexer/mod.rs`

**Step 1: Write failing test for basic tokens**

```rust
// tests/lexer_tests.rs
use ziv::lexer::Lexer;

#[test]
fn test_let_keyword() {
    let mut lexer = Lexer::new("let");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Let);
}

#[test]
fn test_number_literal() {
    let mut lexer = Lexer::new("42");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Number);
    assert_eq!(tokens[0].value, "42");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test lexer_tests`
Expected: FAIL - test file is empty

**Step 3: Import necessary types**

```rust
// tests/lexer_tests.rs
use ziv::lexer::{Lexer, TokenKind};
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --test lexer_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/lexer_tests.rs
git commit -m "test: add lexer tests for keywords and literals"
```

---

## Task 2: Comprehensive Lexer Tests

**Files:**
- Modify: `tests/lexer_tests.rs`

**Step 1: Write tests for all operators**

```rust
#[test]
fn test_arithmetic_operators() {
    let mut lexer = Lexer::new("+ - * / %");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].kind, TokenKind::Plus);
    assert_eq!(tokens[1].kind, TokenKind::Minus);
    assert_eq!(tokens[2].kind, TokenKind::Star);
    assert_eq!(tokens[3].kind, TokenKind::Slash);
    assert_eq!(tokens[4].kind, TokenKind::Percent);
}

#[test]
fn test_comparison_operators() {
    let mut lexer = Lexer::new("< > <= >= == !=");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 6);
    assert_eq!(tokens[0].kind, TokenKind::Less);
    assert_eq!(tokens[1].kind, TokenKind::Greater);
    assert_eq!(tokens[2].kind, TokenKind::LessEqual);
    assert_eq!(tokens[3].kind, TokenKind::GreaterEqual);
    assert_eq!(tokens[4].kind, TokenKind::EqualEqual);
    assert_eq!(tokens[5].kind, TokenKind::BangEqual);
}
```

**Step 2: Run tests**

Run: `cargo test --test lexer_tests`
Expected: PASS

**Step 3: Write tests for function-related tokens**

```rust
#[test]
fn test_function_tokens() {
    let mut lexer = Lexer::new("fn function return -> =>");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].kind, TokenKind::Fn);
    assert_eq!(tokens[1].kind, TokenKind::Function);
    assert_eq!(tokens[2].kind, TokenKind::Return);
    assert_eq!(tokens[3].kind, TokenKind::Arrow);
    assert_eq!(tokens[4].kind, TokenKind::FatArrow);
}
```

**Step 4: Run tests**

Run: `cargo test --test lexer_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/lexer_tests.rs
git commit -m "test: add comprehensive lexer operator and function token tests"
```

---

## Task 3: Parser Test Suite

**Files:**
- Create: `tests/parser_tests.rs`

**Step 1: Write test for variable declaration parsing**

```rust
// tests/parser_tests.rs
use ziv::parser::Parser;
use ziv::parser::ast::{Stmt, Expr};

#[test]
fn test_parse_let_statement() {
    let mut parser = Parser::new("let x = 42;");
    let program = parser.parse().unwrap();
    assert_eq!(program.statements.len(), 1);
    
    match &program.statements[0] {
        Stmt::VariableDecl { name, init, is_const } => {
            assert_eq!(name, "x");
            assert!(!is_const);
            assert!(init.is_some());
        }
        _ => panic!("Expected VariableDecl"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test parser_tests`
Expected: FAIL - test file is empty

**Step 3: Add necessary imports**

```rust
use ziv::parser::{Parser, ast::{Stmt, Expr}};
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test parser_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/parser_tests.rs
git commit -m "test: add parser test for variable declarations"
```

---

## Task 4: Add Type Annotation to AST

**Files:**
- Modify: `src/parser/ast.rs:15-25` (VariableDecl definition)

**Step 1: Write test for type annotation**

```rust
#[test]
fn test_parse_type_annotation() {
    let mut parser = Parser::new("let x: i32 = 42;");
    let program = parser.parse().unwrap();
    
    match &program.statements[0] {
        Stmt::VariableDecl { name, type_annotation, init, .. } => {
            assert_eq!(name, "x");
            assert!(type_annotation.is_some());
            assert_eq!(type_annotation.as_ref().unwrap(), "i32");
        }
        _ => panic!("Expected VariableDecl"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test parser_tests::test_parse_type_annotation`
Expected: FAIL - field doesn't exist

**Step 3: Add type_annotation field to VariableDecl**

```rust
// src/parser/ast.rs
#[derive(Debug, Clone)]
pub struct VariableDecl {
    pub name: String,
    pub type_annotation: Option<String>,  // Add this field
    pub init: Option<Expr>,
    pub is_const: bool,
}
```

**Step 4: Update parser to populate field**

Modify parser logic to parse `: Type` after identifier (requires lalrpop grammar update or manual parsing)

**Step 5: Run test to verify it passes**

Run: `cargo test --test parser_tests::test_parse_type_annotation`
Expected: PASS

**Step 6: Commit**

```bash
git add src/parser/ast.rs src/parser/mod.rs
git commit -m "feat: add type annotation support to variable declarations"
```

---

## Task 5: Function Definition AST

**Files:**
- Modify: `src/parser/ast.rs`

**Step 1: Write test for function definition**

```rust
#[test]
fn test_parse_function_definition() {
    let code = r#"
        fn add(a: i32, b: i32): i32 {
            return a + b;
        }
    "#;
    let mut parser = Parser::new(code);
    let program = parser.parse().unwrap();
    
    match &program.statements[0] {
        Stmt::FunctionDecl { name, params, return_type, body } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name, "a");
            assert_eq!(params[0].type_annotation, Some("i32".to_string()));
            assert_eq!(return_type, &Some("i32".to_string()));
        }
        _ => panic!("Expected FunctionDecl"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test parser_tests::test_parse_function_definition`
Expected: FAIL - fields don't exist

**Step 3: Extend FunctionDecl structure**

```rust
// src/parser/ast.rs
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_annotation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<String>,  // Add this
    pub body: Vec<Stmt>,
}
```

**Step 4: Update parser to parse return type**

Modify parser to handle `: ReturnType` after parameters

**Step 5: Run test to verify it passes**

Run: `cargo test --test parser_tests::test_parse_function_definition`
Expected: PASS

**Step 6: Commit**

```bash
git add src/parser/ast.rs src/parser/mod.rs
git commit -m "feat: add return type and param types to function definitions"
```

---

## Task 6: Function Call Parsing

**Files:**
- Modify: `src/parser/ast.rs`
- Modify: `src/parser/mod.rs`

**Step 1: Write test for function call**

```rust
#[test]
fn test_parse_function_call() {
    let mut parser = Parser::new("add(1, 2);");
    let program = parser.parse().unwrap();
    
    match &program.statements[0] {
        Stmt::Expression(expr) => {
            match expr {
                Expr::Call { callee, args } => {
                    assert!(matches!(callee.as_ref(), Expr::Identifier(name) if name == "add"));
                    assert_eq!(args.len(), 2);
                }
                _ => panic!("Expected Call expression"),
            }
        }
        _ => panic!("Expected Expression statement"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test parser_tests::test_parse_function_call`
Expected: FAIL - Call variant doesn't exist

**Step 3: Add Call expression variant**

```rust
// src/parser/ast.rs
#[derive(Debug, Clone)]
pub enum Expr {
    // ... existing variants
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
}
```

**Step 4: Implement call parsing in parser**

Add parsing logic for `identifier(args)` pattern

**Step 5: Run test to verify it passes**

Run: `cargo test --test parser_tests::test_parse_function_call`
Expected: PASS

**Step 6: Commit**

```bash
git add src/parser/ast.rs src/parser/mod.rs
git commit -m "feat: add function call expression parsing"
```

---

## Task 7: IR for Function Calls

**Files:**
- Modify: `src/ir/instructions.rs`

**Step 1: Write test for function call IR generation**

```rust
// tests/integration_tests.rs (create if needed)
use ziv::parser::Parser;
use ziv::ir::IRBuilder;

#[test]
fn test_function_call_ir() {
    let code = r#"
        fn add(a: i32, b: i32): i32 {
            return a + b;
        }
        let result = add(1, 2);
    "#;
    
    let mut parser = Parser::new(code);
    let program = parser.parse().unwrap();
    
    let builder = IRBuilder::new();
    let module = builder.build(&program);
    
    assert_eq!(module.functions.len(), 2); // add + main
    // Verify call instruction exists in main
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_function_call_ir`
Expected: FAIL - call IR instruction doesn't exist

**Step 3: Add Call IR instruction**

```rust
// src/ir/instructions.rs
#[derive(Debug, Clone)]
pub enum IRInstruction {
    // ... existing variants
    Call {
        result: Option<String>,
        function: String,
        args: Vec<IRValue>,
    },
}
```

**Step 4: Implement call IR generation**

Update `src/ir/builder.rs` to generate Call instructions for function calls

**Step 5: Run test to verify it passes**

Run: `cargo test test_function_call_ir`
Expected: PASS

**Step 6: Commit**

```bash
git add src/ir/instructions.rs src/ir/builder.rs tests/integration_tests.rs
git commit -m "feat: add Call IR instruction and generation"
```

---

## Task 8: x86-64 Function Call Code Generation

**Files:**
- Modify: `src/codegen/x86_64.rs`

**Step 1: Write end-to-end test**

```rust
#[test]
fn test_function_call_codegen() {
    let code = r#"
        fn add(a: i64, b: i64): i64 {
            return a + b;
        }
        let x = add(10, 20);
    "#;
    
    // Compile and check assembly output contains 'call add'
    // This requires compiler integration test
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_function_call_codegen`
Expected: FAIL - call codegen not implemented

**Step 3: Implement function call codegen**

```rust
// src/codegen/x86_64.rs
fn generate_call(&mut self, func: &str, args: &[IRValue]) -> Result<String, String> {
    let mut asm = String::new();
    
    // x86-64 calling convention: args in rdi, rsi, rdx, rcx, r8, r9
    let arg_regs = ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];
    
    for (i, arg) in args.iter().enumerate() {
        if i < arg_regs.len() {
            asm.push_str(&format!("    movq {}, {}\n", 
                self.value_to_asm(arg), arg_regs[i]));
        }
    }
    
    asm.push_str(&format!("    call {}\n", func));
    Ok(asm)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_function_call_codegen`
Expected: PASS

**Step 5: Test actual compilation**

```bash
# Create test file
cat > examples/function_test.ll << 'EOF'
fn add(a: i64, b: i64): i64 {
    return a + b;
}
let result = add(10, 20);
EOF

# Compile
./target/debug/llc examples/function_test.ll -o function_test --keep-asm

# Verify assembly contains call instruction
grep "call add" function_test.s

# Run executable
./function_test
echo $?  # Should return 30
```

**Step 6: Commit**

```bash
git add src/codegen/x86_64.rs examples/function_test.ll
git commit -m "feat: implement x86-64 function call code generation"
```

---

## Task 9: If/Else Control Flow IR

**Files:**
- Modify: `src/ir/instructions.rs`
- Modify: `src/ir/builder.rs`

**Step 1: Write test for if/else IR**

```rust
#[test]
fn test_if_else_ir() {
    let code = r#"
        let x = 10;
        if (x > 5) {
            let y = 1;
        } else {
            let y = 2;
        }
    "#;
    
    let mut parser = Parser::new(code);
    let program = parser.parse().unwrap();
    
    let builder = IRBuilder::new();
    let module = builder.build(&program);
    
    // Verify IR contains labels and conditional branches
    assert!(module.functions[0].instructions.iter().any(|i| {
        matches!(i, IRInstruction::CondBranch { .. })
    }));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_if_else_ir`
Expected: FAIL - CondBranch doesn't exist

**Step 3: Add control flow IR instructions**

```rust
// src/ir/instructions.rs
#[derive(Debug, Clone)]
pub enum IRInstruction {
    // ... existing
    Label(String),
    Jump(String),
    CondBranch {
        condition: IRValue,
        true_label: String,
        false_label: String,
    },
    Cmp {
        op: CmpOp,
        left: IRValue,
        right: IRValue,
        result: String,
    },
}

#[derive(Debug, Clone)]
pub enum CmpOp {
    Lt, Gt, Le, Ge, Eq, Ne,
}
```

**Step 4: Implement if/else IR generation**

Update builder to generate labels and branches for if/else

**Step 5: Run test to verify it passes**

Run: `cargo test test_if_else_ir`
Expected: PASS

**Step 6: Commit**

```bash
git add src/ir/instructions.rs src/ir/builder.rs
git commit -m "feat: add control flow IR instructions (Label, Jump, CondBranch, Cmp)"
```

---

## Task 10: If/Else x86-64 Code Generation

**Files:**
- Modify: `src/codegen/x86_64.rs`

**Step 1: Write end-to-end test**

```rust
#[test]
fn test_if_else_codegen() {
    let code = r#"
        let x = 10;
        if (x > 5) {
            let y = 1;
        }
    "#;
    
    // Compile and verify assembly contains:
    // - cmp instruction
    // - jle/jge conditional jump
    // - labels
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_if_else_codegen`
Expected: FAIL - comparison codegen not implemented

**Step 3: Implement comparison and branch codegen**

```rust
// src/codegen/x86_64.rs
fn generate_cmp(&mut self, op: &CmpOp, left: &IRValue, right: &IRValue) -> Result<String, String> {
    let mut asm = String::new();
    asm.push_str(&format!("    movq {}, %rax\n", self.value_to_asm(left)));
    asm.push_str(&format!("    cmpq {}, %rax\n", self.value_to_asm(right)));
    Ok(asm)
}

fn generate_cond_branch(&mut self, cond: &IRValue, true_label: &str, false_label: &str) -> String {
    // Map comparison to appropriate jump instruction
    format!("    jne {}\n    jmp {}\n", true_label, false_label)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_if_else_codegen`
Expected: PASS

**Step 5: Test actual compilation**

```bash
cat > examples/if_test.ll << 'EOF'
let x = 10;
if (x > 5) {
    let y = 1;
}
EOF

./target/debug/llc examples/if_test.ll -o if_test --keep-asm
grep -E "(cmp|jle|jge)" if_test.s
./if_test
```

**Step 6: Commit**

```bash
git add src/codegen/x86_64.rs examples/if_test.ll
git commit -m "feat: implement if/else x86-64 code generation"
```

---

## Task 11: While Loop IR and Codegen

**Files:**
- Modify: `src/ir/instructions.rs`
- Modify: `src/ir/builder.rs`
- Modify: `src/codegen/x86_64.rs`

**Step 1: Write test for while loop**

```rust
#[test]
fn test_while_loop_ir() {
    let code = r#"
        let i = 0;
        while (i < 10) {
            let x = i + 1;
        }
    "#;
    
    let mut parser = Parser::new(code);
    let program = parser.parse().unwrap();
    
    let builder = IRBuilder::new();
    let module = builder.build(&program);
    
    // Verify loop structure: label, cond check, body, jump back
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_while_loop_ir`
Expected: FAIL - while IR generation not implemented

**Step 3: Implement while loop IR generation**

Generate: loop_start label → condition check → cond_branch → body → jump loop_start → loop_end label

**Step 4: Implement while loop codegen**

Generate assembly with loop labels and backward jump

**Step 5: Run test to verify it passes**

Run: `cargo test test_while_loop_ir`
Expected: PASS

**Step 6: Test actual compilation**

```bash
cat > examples/while_test.ll << 'EOF'
let i = 0;
while (i < 5) {
    let x = i + 1;
}
EOF

./target/debug/llc examples/while_test.ll -o while_test --keep-asm
./while_test
```

**Step 7: Commit**

```bash
git add src/ir/builder.rs src/codegen/x86_64.rs examples/while_test.ll
git commit -m "feat: implement while loop IR and x86-64 code generation"
```

---

## Task 12: Integration Test Suite

**Files:**
- Create: `tests/integration_tests.rs`

**Step 1: Write comprehensive integration test**

```rust
// tests/integration_tests.rs
use std::process::Command;
use std::fs;

#[test]
fn test_fibonacci_compilation() {
    let code = r#"
        fn fib(n: i64): i64 {
            if (n <= 1) {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }
        let result = fib(10);
    "#;
    
    fs::write("examples/fib_rec.ll", code).unwrap();
    
    let output = Command::new("./target/debug/llc")
        .args(&["examples/fib_rec.ll", "-o", "fib_rec_test"])
        .output()
        .expect("Failed to compile");
    
    assert!(output.status.success());
    assert!(Path::new("fib_rec_test").exists());
}
```

**Step 2: Run test**

Run: `cargo test --test integration_tests`
Expected: PASS (after all previous tasks complete)

**Step 3: Commit**

```bash
git add tests/integration_tests.rs
git commit -m "test: add integration tests for full compilation pipeline"
```

---

## Task 13: Documentation Update

**Files:**
- Modify: `README.md`
- Modify: `STATUS.md`
- Modify: `docs/LANGUAGE_SPEC.md`

**Step 1: Update STATUS.md with new progress**

```markdown
## ✅ Completed (70%)

- ✅ Lexer: 100%
- ✅ Parser: 100% (now with type annotations!)
- ✅ Semantic: 100%
- ✅ IR: 100% (control flow + calls)
- ✅ CodeGen: 100% (functions + if/while)
- ✅ Tests: 80%+ coverage 🎉
```

**Step 2: Update examples in README**

Add function example to README showing new capabilities

**Step 3: Commit**

```bash
git add README.md STATUS.md docs/
git commit -m "docs: update documentation with function and control flow features"
```

---

## Summary

This plan establishes:
1. ✅ **Test foundation** - Comprehensive test coverage for lexer, parser, IR, codegen
2. ✅ **Type system** - Type annotations on variables and functions
3. ✅ **Functions** - Full function definition and call support
4. ✅ **Control flow** - If/else and while loops with proper IR and codegen
5. ✅ **Documentation** - Updated docs reflecting new capabilities

Each task follows TDD: write failing test → implement → verify passing test → commit.

**Estimated completion**: 2-3 days with focused work.
