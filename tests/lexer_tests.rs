use lightlang::lexer::{Lexer, Token};

#[test]
fn test_let_keyword() {
    let mut lexer = Lexer::new("let");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Let);
}

#[test]
fn test_number_literal() {
    let mut lexer = Lexer::new("42");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Number(42));
}

#[test]
fn test_identifier() {
    let mut lexer = Lexer::new("x");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Identifier("x".to_string()));
}

#[test]
fn test_arithmetic_operators() {
    let mut lexer = Lexer::new("+ - * / %");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0], Token::Plus);
    assert_eq!(tokens[1], Token::Minus);
    assert_eq!(tokens[2], Token::Star);
    assert_eq!(tokens[3], Token::Slash);
    assert_eq!(tokens[4], Token::Percent);
}

#[test]
fn test_comparison_operators() {
    let mut lexer = Lexer::new("< > <= >= == !=");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 6);
    assert_eq!(tokens[0], Token::Less);
    assert_eq!(tokens[1], Token::Greater);
    assert_eq!(tokens[2], Token::LessEqual);
    assert_eq!(tokens[3], Token::GreaterEqual);
    assert_eq!(tokens[4], Token::EqualEqual);
    assert_eq!(tokens[5], Token::BangEqual);
}

#[test]
fn test_function_tokens() {
    let mut lexer = Lexer::new("function return -> =>");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0], Token::Function);
    assert_eq!(tokens[1], Token::Return);
    assert_eq!(tokens[2], Token::Arrow);
    assert_eq!(tokens[3], Token::FatArrow);
}
