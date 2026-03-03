//! Abstract Syntax Tree (AST) definitions for LightLang

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Program { statements }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_annotation: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    VariableDecl {
        name: String,
        type_annotation: Option<String>,
        init: Option<Expr>,
        is_const: bool,
    },
    FunctionDecl {
        name: String,
        params: Vec<Param>,
        return_type: Option<String>,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Call {
        callee: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        let num = Literal::Number(42);
        assert_eq!(num, Literal::Number(42));
    }

    #[test]
    fn test_expr() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Number(1))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::Number(2))),
        };
        assert!(matches!(expr, Expr::Binary { .. }));
    }

    #[test]
    fn test_function_decl() {
        let stmt = Stmt::FunctionDecl {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            body: vec![Stmt::Return(Some(Expr::Binary {
                left: Box::new(Expr::Identifier("a".to_string())),
                op: BinaryOp::Add,
                right: Box::new(Expr::Identifier("b".to_string())),
            }))],
        };
        assert!(matches!(stmt, Stmt::FunctionDecl { .. }));
    }
}
