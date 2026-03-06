use ziv::parser::{
    ast::{Expr, Stmt},
    Parser,
};

#[test]
fn test_parse_let_statement() {
    let mut parser = Parser::new("let x = 42;");
    let program = parser.parse().unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Stmt::VariableDecl {
            name,
            type_annotation,
            init,
            is_const,
        } => {
            assert_eq!(name, "x");
            assert!(type_annotation.is_none());
            assert!(!is_const);
            assert!(init.is_some());
        }
        _ => panic!("Expected VariableDecl"),
    }
}

#[test]
fn test_parse_const_statement() {
    let mut parser = Parser::new("const PI = 3.14;");
    let program = parser.parse().unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Stmt::VariableDecl {
            name,
            type_annotation,
            init,
            is_const,
        } => {
            assert_eq!(name, "PI");
            assert!(type_annotation.is_none());
            assert!(is_const);
            assert!(init.is_some());
        }
        _ => panic!("Expected VariableDecl"),
    }
}

#[test]
fn test_parse_binary_expression() {
    let mut parser = Parser::new("let result = 10 + 20;");
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Stmt::VariableDecl { name, init, .. } => {
            assert_eq!(name, "result");
            if let Some(expr) = init {
                match expr {
                    Expr::Binary { left, op, right } => {
                        assert!(matches!(left.as_ref(), Expr::Literal(_)));
                        assert!(matches!(right.as_ref(), Expr::Literal(_)));
                    }
                    _ => panic!("Expected Binary expression"),
                }
            }
        }
        _ => panic!("Expected VariableDecl"),
    }
}
