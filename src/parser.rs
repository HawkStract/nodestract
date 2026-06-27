use crate::engine::lexer::Token;

pub struct Parser {
    pub tokens: Vec<Token>,
    pub position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Stub parsing method that returns an empty Program for now.
    pub fn parse(&mut self) -> crate::engine::ast::Program {
        crate::engine::ast::Program {
            statements: Vec::new(),
        }
    }
}