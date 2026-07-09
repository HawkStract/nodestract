#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl { is_mutable: bool, name: String, value: Expression },
    Assignment { target: Expression, value: Expression },
    IfStatement { condition: Expression, then_branch: Vec<Statement>, else_branch: Option<Vec<Statement>> },
    WhileStatement { condition: Expression, body: Vec<Statement> },
    ForStatement { iterator: String, start: Expression, end: Expression, body: Vec<Statement> },
    SwitchStatement { discriminant: Expression, cases: Vec<(Expression, Vec<Statement>)>, default_case: Option<Vec<Statement>> },
    ReturnStatement { value: Expression },
    CapabilityUse { service: String, params: Vec<String> },
    FunctionDecl { name: String, params: Vec<String>, body: Vec<Statement> },
    TryCatchStatement { try_block: Vec<Statement>, catch_variable: Option<String>, catch_block: Vec<Statement>, finally_block: Option<Vec<Statement>> },
    ThrowStatement { value: Expression },
    Break,
    Continue,
    Expr(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    LiteralStr(String),
    LiteralNum(f64),
    LiteralBool(bool),
    LiteralNull,
    Array(Vec<Expression>),
    Map(Vec<(String, Expression)>),
    Index { target: Box<Expression>, index: Box<Expression> },
    Variable(String),
    BinaryOp { left: Box<Expression>, operator: String, right: Box<Expression> },
    UnaryOp { operator: String, operand: Box<Expression> },
    Ternary { condition: Box<Expression>, true_expr: Box<Expression>, false_expr: Box<Expression> },
    FunctionCall { target: Box<Expression>, args: Vec<Expression> },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}