use crate::ast::{Program, Statement, Expression};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Program {
        let mut statements = Vec::new();

        while self.position < self.tokens.len() {
            let token = &self.tokens[self.position];
            
            match token {
                Token::EOF => break,
                Token::Use => {
                    statements.push(self.parse_capability());
                },
                Token::Lock => {
                    statements.push(self.parse_var_decl(false, false));
                },
                Token::Stract => {
                    statements.push(self.parse_var_decl(true, false));
                },
                Token::Vault => {
                    statements.push(self.parse_var_decl(false, true));
                },
                Token::Func => {
                    statements.push(self.parse_function());
                },
                Token::Module => {
                    self.advance();
                    self.advance();
                },
                _ => {
                    self.advance();
                }
            }
        }

        Program { statements }
    }

    fn parse_capability(&mut self) -> Statement {
        self.advance();
        self.advance();
        
        let service_name = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => "Unknown".to_string(),
        };
        self.advance();

        while self.current_token() != &Token::RightBrace && self.current_token() != &Token::EOF {
            self.advance();
        }
        self.advance();

        Statement::CapabilityUse {
            service: service_name,
            params: vec![],
        }
    }

    fn parse_var_decl(&mut self, is_mutable: bool, is_secure: bool) -> Statement {
        self.advance();
        
        let name = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => "Unknown".to_string(),
        };
        self.advance();

        self.advance();

        let value = match self.current_token() {
            Token::StringLiteral(s) => Expression::LiteralStr(s.clone()),
            _ => Expression::LiteralStr("".to_string()),
        };
        self.advance();

        Statement::VarDecl {
            is_mutable,
            is_secure,
            name,
            value,
        }
    }

    fn parse_function(&mut self) -> Statement {
        self.advance();
        
        let name = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => "Anon".to_string(),
        };
        self.advance();

        while self.current_token() != &Token::LeftBrace {
            self.advance();
        }
        self.advance();

        let mut body = Vec::new();
        while self.current_token() != &Token::RightBrace && self.current_token() != &Token::EOF {
            match self.current_token() {
                Token::Stract => body.push(self.parse_var_decl(true, false)),
                Token::Identifier(_) => {
                    body.push(self.parse_func_call_stmt());
                }
                _ => { self.advance(); }
            }
        }
        self.advance();

        Statement::FunctionDecl { name, body }
    }

    fn parse_func_call_stmt(&mut self) -> Statement {
        let target = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => "".to_string(),
        };
        self.advance();
        self.advance();
        let method = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => "".to_string(),
        };
        self.advance();
        
        while self.current_token() != &Token::RightParen {
            self.advance();
        }
        self.advance();

        Statement::Expr(Expression::FunctionCall {
            target: format!("{}.{}", target, method),
            args: vec![],
        })
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn current_token(&self) -> &Token {
        if self.position >= self.tokens.len() {
            return &Token::EOF;
        }
        &self.tokens[self.position]
    }
}