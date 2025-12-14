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

        let value = self.parse_expression();

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
                Token::Stract | Token::Lock | Token::Vault => {
                    let is_mut = match self.current_token() {
                        Token::Stract => true,
                        _ => false,
                    };
                    let is_sec = match self.current_token() {
                        Token::Vault => true,
                        _ => false,
                    };
                    body.push(self.parse_var_decl(is_mut, is_sec));
                }
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
        
        self.advance();

        let mut args = Vec::new();
        if self.current_token() != &Token::RightParen {
            args.push(self.parse_expression());
            while self.current_token() == &Token::Comma {
                self.advance();
                args.push(self.parse_expression());
            }
        }

        self.advance();

        Statement::Expr(Expression::FunctionCall {
            target: format!("{}.{}", target, method),
            args,
        })
    }

    fn parse_expression(&mut self) -> Expression {
        let mut left = self.parse_primary();

        while self.current_token() == &Token::Plus {
            self.advance();
            let right = self.parse_primary();
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: "+".to_string(),
                right: Box::new(right),
            };
        }

        left
    }

    fn parse_primary(&mut self) -> Expression {
        match self.current_token() {
            Token::StringLiteral(s) => {
                let val = s.clone();
                self.advance();
                Expression::LiteralStr(val)
            }
            Token::Number(n) => {
                let val = *n;
                self.advance();
                Expression::LiteralNum(val)
            }
            Token::Identifier(s) => {
                let val = s.clone();
                self.advance();
                Expression::Variable(val)
            }
            _ => {
                self.advance();
                Expression::LiteralStr("".to_string())
            }
        }
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