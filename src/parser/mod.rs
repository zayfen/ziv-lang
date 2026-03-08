//! Parser implementation

pub mod ast;
use crate::lexer::Token;
use ast::*;

pub type ParseResult<T> = Result<T, String>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let mut lexer = crate::lexer::Lexer::new(source);
        let tokens = lexer.tokenize().unwrap_or_else(|_| vec![Token::EOF]);
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let Some(stmt) = self.parse_stmt()? {
                statements.push(stmt);
            }
        }

        Ok(Program::new(statements))
    }

    fn parse_stmt(&mut self) -> ParseResult<Option<Stmt>> {
        match self.peek() {
            Some(Token::Let) | Some(Token::Const) => self.parse_var_decl(),

            Some(Token::Function) => self.parse_function_decl(),

            Some(Token::Return) => self.parse_return_stmt(),

            Some(Token::If) => self.parse_if_stmt(),

            Some(Token::While) => self.parse_while_stmt(),

            Some(Token::LeftBrace) => self.parse_block(),

            _ => {
                let expr = self.parse_expr()?;

                if self.match_token(&Token::Equal) {
                    self.advance();
                    let value = self.parse_expr()?;
                    self.consume_semicolon()?;

                    if let Expr::Identifier(name) = expr {
                        Ok(Some(Stmt::Assignment { name, value }))
                    } else {
                        Err("Invalid assignment target".to_string())
                    }
                } else {
                    self.consume_semicolon()?;
                    Ok(Some(Stmt::Expression(expr)))
                }
            }
        }
    }

    fn parse_var_decl(&mut self) -> ParseResult<Option<Stmt>> {
        let is_const = matches!(self.peek(), Some(Token::Const));
        self.advance(); // consume let/const

        let name = self.consume_identifier()?;

        let type_annotation = if self.match_token(&Token::Colon) {
            self.advance();
            Some(self.consume_identifier()?)
        } else {
            None
        };

        let init = if self.match_token(&Token::Equal) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.consume_semicolon()?;
        Ok(Some(Stmt::VariableDecl {
            name,
            type_annotation,
            init,
            is_const,
        }))
    }

    fn parse_function_decl(&mut self) -> ParseResult<Option<Stmt>> {
        self.advance();

        let name = self.consume_identifier()?;

        self.consume(&Token::LeftParen)?;
        let mut params = Vec::new();
        while !self.match_token(&Token::RightParen) {
            let param_name = self.consume_identifier()?;

            let type_annotation = if self.match_token(&Token::Colon) {
                self.advance();
                Some(self.consume_identifier()?)
            } else {
                None
            };

            params.push(Param {
                name: param_name,
                type_annotation,
            });

            if !self.match_token(&Token::Comma) {
                break;
            }
            self.advance();
        }
        self.consume(&Token::RightParen)?;

        let return_type = if self.match_token(&Token::Colon) {
            self.advance();
            Some(self.consume_identifier()?)
        } else {
            None
        };

        let body = self.parse_block_internal()?;

        Ok(Some(Stmt::FunctionDecl {
            name,
            params,
            return_type,
            body,
        }))
    }

    fn parse_return_stmt(&mut self) -> ParseResult<Option<Stmt>> {
        self.advance(); // consume 'return'

        let value = if self.match_token(&Token::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };

        self.consume_semicolon()?;
        Ok(Some(Stmt::Return(value)))
    }

    fn parse_if_stmt(&mut self) -> ParseResult<Option<Stmt>> {
        self.advance(); // consume 'if'

        self.consume(&Token::LeftParen)?;
        let condition = self.parse_expr()?;
        self.consume(&Token::RightParen)?;

        let then_branch = self.parse_block_internal()?;

        let else_branch = if self.match_token(&Token::Else) {
            self.advance();
            Some(self.parse_block_internal()?)
        } else {
            None
        };

        Ok(Some(Stmt::If {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn parse_while_stmt(&mut self) -> ParseResult<Option<Stmt>> {
        self.advance(); // consume 'while'

        self.consume(&Token::LeftParen)?;
        let condition = self.parse_expr()?;
        self.consume(&Token::RightParen)?;

        let body = self.parse_block_internal()?;

        Ok(Some(Stmt::While { condition, body }))
    }

    fn parse_block(&mut self) -> ParseResult<Option<Stmt>> {
        let stmts = self.parse_block_internal()?;
        Ok(Some(Stmt::Block(stmts)))
    }

    fn parse_block_internal(&mut self) -> ParseResult<Vec<Stmt>> {
        self.consume(&Token::LeftBrace)?;

        let mut statements = Vec::new();
        while !self.match_token(&Token::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.parse_stmt()? {
                statements.push(stmt);
            }
        }

        self.consume(&Token::RightBrace)?;
        Ok(statements)
    }

    fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_addition()?;

        while let Some(_) = self.match_comparison() {
            let op = self.consume_comparison()?;
            let right = self.parse_addition()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_addition(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_multiplication()?;

        while let Some(_) = self.match_addition() {
            let op = self.consume_addition()?;
            let right = self.parse_multiplication()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplication(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_unary()?;

        while let Some(_) = self.match_multiplication() {
            let op = self.consume_multiplication()?;
            let right = self.parse_unary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> ParseResult<Expr> {
        if let Some(Token::Minus) = self.peek() {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(-1))),
                op: BinaryOp::Mul,
                right: Box::new(expr),
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> ParseResult<Expr> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Number(n)))
            }
            Some(Token::Float(f)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Float(f)))
            }
            Some(Token::String(s)) => {
                self.advance();
                Ok(Expr::Literal(Literal::String(s)))
            }
            Some(Token::Boolean(b)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(b)))
            }
            Some(Token::Identifier(name)) => {
                self.advance();

                // Check if it's a function call
                if self.match_token(&Token::LeftParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.match_token(&Token::RightParen) {
                        args.push(self.parse_expr()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                    self.consume(&Token::RightParen)?;
                    Ok(Expr::Call { callee: name, args })
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            Some(Token::LeftParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.consume(&Token::RightParen)?;
                Ok(expr)
            }
            _ => Err(format!("Unexpected token: {:?}", self.peek())),
        }
    }

    // Helper methods
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Some(Token::EOF))
    }

    fn match_token(&self, token: &Token) -> bool {
        matches!(self.peek(), Some(t) if t == token)
    }

    fn consume(&mut self, token: &Token) -> ParseResult<()> {
        if self.match_token(token) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", token, self.peek()))
        }
    }

    fn consume_identifier(&mut self) -> ParseResult<String> {
        match self.peek() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(format!("Expected identifier, got {:?}", self.peek())),
        }
    }

    fn consume_semicolon(&mut self) -> ParseResult<()> {
        if self.match_token(&Token::Semicolon) {
            self.advance();
        }
        Ok(())
    }

    fn match_comparison(&self) -> Option<()> {
        match self.peek() {
            Some(Token::EqualEqual)
            | Some(Token::BangEqual)
            | Some(Token::Less)
            | Some(Token::LessEqual)
            | Some(Token::Greater)
            | Some(Token::GreaterEqual) => Some(()),
            _ => None,
        }
    }

    fn consume_comparison(&mut self) -> ParseResult<BinaryOp> {
        match self.peek() {
            Some(Token::EqualEqual) => {
                self.advance();
                Ok(BinaryOp::Eq)
            }
            Some(Token::BangEqual) => {
                self.advance();
                Ok(BinaryOp::Ne)
            }
            Some(Token::Less) => {
                self.advance();
                Ok(BinaryOp::Lt)
            }
            Some(Token::LessEqual) => {
                self.advance();
                Ok(BinaryOp::Le)
            }
            Some(Token::Greater) => {
                self.advance();
                Ok(BinaryOp::Gt)
            }
            Some(Token::GreaterEqual) => {
                self.advance();
                Ok(BinaryOp::Ge)
            }
            _ => Err("Expected comparison operator".to_string()),
        }
    }

    fn match_addition(&self) -> Option<()> {
        match self.peek() {
            Some(Token::Plus) | Some(Token::Minus) => Some(()),
            _ => None,
        }
    }

    fn consume_addition(&mut self) -> ParseResult<BinaryOp> {
        match self.peek() {
            Some(Token::Plus) => {
                self.advance();
                Ok(BinaryOp::Add)
            }
            Some(Token::Minus) => {
                self.advance();
                Ok(BinaryOp::Sub)
            }
            _ => Err("Expected addition operator".to_string()),
        }
    }

    fn match_multiplication(&self) -> Option<()> {
        match self.peek() {
            Some(Token::Star) | Some(Token::Slash) | Some(Token::Percent) => Some(()),
            _ => None,
        }
    }

    fn consume_multiplication(&mut self) -> ParseResult<BinaryOp> {
        match self.peek() {
            Some(Token::Star) => {
                self.advance();
                Ok(BinaryOp::Mul)
            }
            Some(Token::Slash) => {
                self.advance();
                Ok(BinaryOp::Div)
            }
            _ => Err("Expected multiplication operator".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let mut parser = Parser::new("42;");
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_var_decl() {
        let mut parser = Parser::new("let x = 42;");
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_assignment() {
        let mut parser = Parser::new("let x = 1; x = 2;");
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 2);
        assert!(matches!(program.statements[1], Stmt::Assignment { .. }));
    }

    #[test]
    fn test_parse_function_if_else_while_and_call() {
        let src = r#"
            function add(a: int, b: int): int {
                if (a == b) { return a; } else { return b; }
            }
            while (1) { add(1, 2); }
        "#;
        let mut parser = Parser::new(src);
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 2);
        assert!(matches!(program.statements[0], Stmt::FunctionDecl { .. }));
        assert!(matches!(program.statements[1], Stmt::While { .. }));
    }

    #[test]
    fn test_parse_unary_grouping_and_comparisons() {
        let mut parser = Parser::new("let x = -(1 + 2) * 3 < 10;");
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(
            &program.statements[0],
            Stmt::VariableDecl {
                init: Some(Expr::Binary { .. }),
                ..
            }
        ));
    }

    #[test]
    fn test_parse_block_statement() {
        let mut parser = Parser::new("{ let x = 1; return x; }");
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::Block(_)));
    }

    #[test]
    fn test_parse_return_without_value() {
        let mut parser = Parser::new("function f() { return; }");
        let program = parser.parse().unwrap();
        assert!(matches!(program.statements[0], Stmt::FunctionDecl { .. }));
    }

    #[test]
    fn test_parse_invalid_assignment_target_error() {
        let mut parser = Parser::new("(1 + 2) = 3;");
        let err = parser.parse().unwrap_err();
        assert!(err.contains("Invalid assignment target"));
    }

    #[test]
    fn test_parse_missing_identifier_error() {
        let mut parser = Parser::new("let = 1;");
        let err = parser.parse().unwrap_err();
        assert!(err.contains("Expected identifier"));
    }

    #[test]
    fn test_parse_missing_paren_error() {
        let mut parser = Parser::new("if (1 { return 1; }");
        let err = parser.parse().unwrap_err();
        assert!(err.contains("Expected RightParen"));
    }

    #[test]
    fn test_parse_unexpected_primary_error() {
        let mut parser = Parser::new(";");
        let err = parser.parse().unwrap_err();
        assert!(err.contains("Unexpected token"));
    }

    #[test]
    fn test_parse_percent_operator_hits_error_path() {
        let mut parser = Parser::new("let x = 5 % 2;");
        let err = parser.parse().unwrap_err();
        assert!(err.contains("Expected multiplication operator"));
    }

    #[test]
    fn test_parse_typed_decl_without_init_and_if_without_else() {
        let mut parser = Parser::new("let x: int; if (1) { x = 2; }");
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 2);
    }

    #[test]
    fn test_parse_float_string_boolean_and_more_ops() {
        let mut parser = Parser::new(
            r#"
            let a = 1.5;
            let b = "s";
            let c = true;
            let d = 8 / 2 - 1;
            let e = 1 != 2;
            let f = 1 <= 2;
            let g = 2 > 1;
            let h = 2 >= 1;
            "#,
        );
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 8);
    }

    #[test]
    fn test_direct_operator_consumers_error_paths() {
        let mut parser = Parser::new(";");
        let cmp_err = parser.consume_comparison().unwrap_err();
        assert!(cmp_err.contains("Expected comparison operator"));

        let add_err = parser.consume_addition().unwrap_err();
        assert!(add_err.contains("Expected addition operator"));

        let mul_err = parser.consume_multiplication().unwrap_err();
        assert!(mul_err.contains("Expected multiplication operator"));
    }

    #[test]
    fn test_parser_new_falls_back_to_eof_on_lexer_error() {
        let source = format!("let x = {};", "9".repeat(200));
        let mut parser = Parser::new(&source);
        let program = parser.parse().unwrap();
        assert!(program.statements.is_empty());
    }
}
