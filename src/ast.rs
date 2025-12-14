#[allow(dead_code)]
#[derive(Debug)]
pub enum Statement {
    VarDecl {
        is_mutable: bool,
        is_secure: bool,
        name: String,
        value: Expression,
    },
    CapabilityUse {
        service: String,
        params: Vec<(String, String)>,
    },
    FunctionDecl {
        name: String,
        body: Vec<Statement>,
    },
    Expr(Expression),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Expression {
    LiteralStr(String),
    LiteralNum(f64),
    Variable(String),
    BinaryOp {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    FunctionCall {
        target: String,
        args: Vec<Expression>,
    },
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}