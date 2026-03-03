//! AST definitions

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
pub enum Stmt {
    Expression(Expr),
    VariableDecl {
        name: String,
        init: Option<Expr>,
        is_const: bool,
    },
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
}
