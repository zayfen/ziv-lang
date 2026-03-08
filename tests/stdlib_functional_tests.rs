use tempfile::tempdir;
use ziv::compiler::Compiler;
use ziv::ir::{IRBuilder, IRInstruction};
use ziv::parser::Parser;
use ziv::semantic::SemanticAnalyzer;
use ziv::stdlib::Stdlib;

fn parse(source: &str) -> ziv::parser::ast::Program {
    let mut parser = Parser::new(source);
    parser.parse().unwrap()
}

#[test]
fn test_stdlib_registry_surface_is_complete() {
    let stdlib = Stdlib::new();
    let expected = [
        "print", "println", "read", "eprint", "eprintln", "abs", "min", "max", "sqrt", "pow",
        "floor", "ceil", "round", "strlen", "concat", "substr", "char_at", "to_upper",
        "to_lower", "trim", "contains", "push", "pop", "arrlen", "get", "set", "first", "last",
        "reverse",
    ];

    for name in expected {
        assert!(stdlib.is_builtin(name), "missing builtin: {name}");
    }
}

#[test]
fn test_semantic_accepts_cross_module_stdlib_calls() {
    let program = parse(
        r#"
        print(1);
        println(2);
        read();
        eprint(3);
        eprintln(4);
        abs(5);
        min(1, 2);
        max(1, 2);
        sqrt(9);
        pow(2, 10);
        floor(7);
        ceil(8);
        round(9);
        strlen("abc");
        concat("a", "b");
        substr("abc", 0, 2);
        char_at("abc", 1);
        to_upper("abc");
        to_lower("ABC");
        trim(" x ");
        contains("abc", "b");
        push(0, 1);
        pop(0);
        arrlen(0);
        get(0, 0);
        set(0, 0, 1);
        first(0);
        last(0);
        reverse(0);
        "#,
    );

    let mut analyzer = SemanticAnalyzer::new();
    assert!(analyzer.analyze(&program).is_ok());
}

#[test]
fn test_ir_builder_lowers_print_calls_and_skips_other_builtins() {
    let program = parse(
        r#"
        print(1);
        println("x");
        abs(2);
        strlen("abc");
        push(0, 1);
        "#,
    );

    let module = IRBuilder::new().build(&program);
    let main = module
        .functions
        .iter()
        .find(|func| func.name == "_user_main")
        .unwrap();

    let has_runtime_print = main.instructions.iter().any(|instr| match instr {
        IRInstruction::Call { function, .. } => function == "ziv_print_i64",
        _ => false,
    });
    let has_runtime_println_str = main.instructions.iter().any(|instr| match instr {
        IRInstruction::Call { function, .. } => function == "ziv_println_str",
        _ => false,
    });
    let has_skipped_builtin_call = main.instructions.iter().any(|instr| match instr {
        IRInstruction::Call { function, .. } => {
            matches!(function.as_str(), "abs" | "strlen" | "push")
        }
        _ => false,
    });
    assert!(has_runtime_print);
    assert!(has_runtime_println_str);
    assert!(!has_skipped_builtin_call);
}

#[test]
fn test_ir_builder_preserves_shadowed_builtin_calls() {
    let program = parse(
        r#"
        function print(x) { return x; }
        function abs(x) { return x; }
        print(1);
        abs(2);
        println(3);
        "#,
    );

    let module = IRBuilder::new().build(&program);
    let main = module
        .functions
        .iter()
        .find(|func| func.name == "_user_main")
        .unwrap();

    let has_user_print = main.instructions.iter().any(|instr| {
        matches!(
            instr,
            IRInstruction::Call { function, .. } if function == "print"
        )
    });
    let has_user_abs = main.instructions.iter().any(|instr| {
        matches!(
            instr,
            IRInstruction::Call { function, .. } if function == "abs"
        )
    });
    let has_builtin_println = main.instructions.iter().any(|instr| {
        matches!(
            instr,
            IRInstruction::Call { function, .. } if function == "println"
        )
    });
    let has_runtime_println = main.instructions.iter().any(|instr| {
        matches!(
            instr,
            IRInstruction::Call { function, .. } if function == "ziv_println_i64"
        )
    });

    assert!(has_user_print);
    assert!(has_user_abs);
    assert!(!has_builtin_println);
    assert!(has_runtime_println);
}

#[test]
fn test_compiler_can_compile_program_with_stdlib_calls() {
    let dir = tempdir().unwrap();
    let output = dir.path().join("stdlib_ok");
    let output_str = output.to_string_lossy().to_string();

    let mut compiler = Compiler::new().output(&output_str);
    compiler
        .compile(
            r#"
            print(1);
            println(2);
            abs(3);
            strlen("abc");
            push(0, 1);
            "#,
        )
        .unwrap();

    assert!(output.exists());
}
