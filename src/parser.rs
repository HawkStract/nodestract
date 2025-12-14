use crate::lexer::Token;

// Dichiariamo i moduli figli che si trovano nella cartella src/parser/
pub mod statement;
pub mod expression;

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

    // Funzioni di utilità condivise
    pub fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    pub fn current_token(&self) -> &Token {
        if self.position >= self.tokens.len() {
            return &Token::EOF;
        }
        &self.tokens[self.position]
    }
    
    pub fn peek(&self) -> &Token {
        if self.position + 1 >= self.tokens.len() {
            return &Token::EOF;
        }
        &self.tokens[self.position + 1]
    }
}