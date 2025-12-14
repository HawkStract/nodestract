#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl {
        is_mutable: bool,
        is_secure: bool,
        name: String,
        value: Expression,
    },
    Assignment {
        name: String,
        value: Expression,
    },
    IfStatement {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    WhileStatement {
        condition: Expression,
        body: Vec<Statement>,
    },
    ForStatement {
        iterator: String,
        start: Expression,
        end: Expression,
        body: Vec<Statement>,
    },
    ReturnStatement {
        value: Expression,
    },
    CapabilityUse {
        service: String,
        params: Vec<(String, String)>,
    },
    FunctionDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    Expr(Expression),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Expression {
    LiteralStr(String),
    LiteralNum(f64),
    // NUOVO: Array definition [1, 2, 3]
    Array(Vec<Expression>),
    // NUOVO: Index Access var[0]
    Index {
        target: Box<Expression>,
        index: Box<Expression>,
    },
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

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}