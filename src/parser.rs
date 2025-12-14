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
                Token::Use => { statements.push(self.parse_capability()); },
                Token::Lock => { statements.push(self.parse_var_decl(false, false)); },
                Token::Stract => { statements.push(self.parse_var_decl(true, false)); },
                Token::Vault => { statements.push(self.parse_var_decl(false, true)); },
                Token::Func => { statements.push(self.parse_function()); },
                Token::Module => { self.advance(); self.advance(); },
                _ => { self.advance(); }
            }
        }

        Program { statements }
    }

    fn parse_capability(&mut self) -> Statement {
        self.advance(); self.advance();
        let service_name = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => "Unknown".to_string(),
        };
        self.advance();
        while self.current_token() != &Token::RightBrace && self.current_token() != &Token::EOF {
            self.advance();
        }
        self.advance();
        Statement::CapabilityUse { service: service_name, params: vec![] }
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
        Statement::VarDecl { is_mutable, is_secure, name, value }
    }

    fn parse_function(&mut self) -> Statement {
        self.advance();
        let name = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => "Anon".to_string(),
        };
        self.advance();
        
        self.advance(); 
        let mut params = Vec::new();
        if self.current_token() != &Token::RightParen {
            if let Token::Identifier(param_name) = self.current_token() {
                params.push(param_name.clone());
                self.advance();
            }
            while self.current_token() == &Token::Comma {
                self.advance();
                if let Token::Identifier(param_name) = self.current_token() {
                    params.push(param_name.clone());
                    self.advance();
                }
            }
        }
        self.advance(); 

        self.advance(); 

        let body = self.parse_block();
        Statement::FunctionDecl { name, params, body }
    }

    fn parse_block(&mut self) -> Vec<Statement> {
        let mut body = Vec::new();
        while self.current_token() != &Token::RightBrace && self.current_token() != &Token::EOF {
            match self.current_token() {
                Token::Stract | Token::Lock | Token::Vault => {
                    let is_mut = matches!(self.current_token(), Token::Stract);
                    let is_sec = matches!(self.current_token(), Token::Vault);
                    body.push(self.parse_var_decl(is_mut, is_sec));
                }
                Token::If => {
                    body.push(self.parse_if_statement());
                }
                Token::While => {
                    body.push(self.parse_while_statement());
                }
                Token::For => {
                    body.push(self.parse_for_statement());
                }
                Token::Return => {
                    body.push(self.parse_return_statement());
                }
                Token::Identifier(_) => {
                    if self.peek() == &Token::Equal {
                        body.push(self.parse_assignment());
                    } else {
                        body.push(self.parse_func_call_stmt());
                    }
                }
                _ => { self.advance(); }
            }
        }
        self.advance();
        body
    }

    fn parse_return_statement(&mut self) -> Statement {
        self.advance(); 
        let value = self.parse_expression();
        Statement::ReturnStatement { value }
    }

    fn parse_assignment(&mut self) -> Statement {
        let name = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => "Unknown".to_string(),
        };
        self.advance(); 
        self.advance(); 
        let value = self.parse_expression();
        Statement::Assignment { name, value }
    }

    fn parse_if_statement(&mut self) -> Statement {
        self.advance(); 
        let condition = self.parse_expression();
        
        while self.current_token() != &Token::LeftBrace { self.advance(); }
        self.advance(); 
        
        let then_branch = self.parse_block();
        
        let mut else_branch = None;
        if self.current_token() == &Token::Else {
            self.advance(); 
            while self.current_token() != &Token::LeftBrace { self.advance(); }
            self.advance(); 
            else_branch = Some(self.parse_block());
        }

        Statement::IfStatement { condition, then_branch, else_branch }
    }

    fn parse_while_statement(&mut self) -> Statement {
        self.advance();
        let condition = self.parse_expression();
        while self.current_token() != &Token::LeftBrace { self.advance(); }
        self.advance();
        let body = self.parse_block();
        Statement::WhileStatement { condition, body }
    }

    fn parse_for_statement(&mut self) -> Statement {
        self.advance();
        let iterator = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => "i".to_string(),
        };
        self.advance(); 
        self.advance(); 
        let start = self.parse_primary();
        self.advance(); 
        let end = self.parse_primary();
        
        while self.current_token() != &Token::LeftBrace { self.advance(); }
        self.advance();
        let body = self.parse_block();

        Statement::ForStatement { iterator, start, end, body }
    }

    fn parse_func_call_stmt(&mut self) -> Statement {
        let mut target = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => "".to_string(),
        };
        self.advance(); 

        if self.current_token() == &Token::Dot {
            self.advance(); 
            let method = match self.current_token() {
                Token::Identifier(s) => s.clone(),
                _ => "".to_string(),
            };
            self.advance(); 
            target = format!("{}.{}", target, method);
        }

        if self.current_token() == &Token::LeftParen {
            self.advance();
        }

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
            target,
            args,
        })
    }

    fn parse_expression(&mut self) -> Expression {
        let mut left = self.parse_term();

        while matches!(self.current_token(), Token::Plus | Token::Minus | Token::EqualEqual | Token::Greater | Token::Less) {
            let operator = match self.current_token() {
                Token::Plus => "+".to_string(),
                Token::Minus => "-".to_string(),
                Token::EqualEqual => "==".to_string(),
                Token::Greater => ">".to_string(),
                Token::Less => "<".to_string(),
                _ => "".to_string(),
            };
            self.advance();
            let right = self.parse_term();
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_term(&mut self) -> Expression {
        let mut left = self.parse_primary();
        while matches!(self.current_token(), Token::Star | Token::Slash) {
            let operator = match self.current_token() {
                Token::Star => "*".to_string(),
                Token::Slash => "/".to_string(),
                _ => "".to_string(),
            };
            self.advance();
            let right = self.parse_primary();
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_primary(&mut self) -> Expression {
        match self.current_token() {
            Token::StringLiteral(s) => {
                let val = s.clone(); self.advance();
                Expression::LiteralStr(val)
            }
            Token::Number(n) => {
                let val = *n; self.advance();
                Expression::LiteralNum(val)
            }
            Token::Identifier(s) => {
                let mut name = s.clone();
                self.advance();

                if self.current_token() == &Token::Dot {
                    self.advance();
                    if let Token::Identifier(method) = self.current_token() {
                         name = format!("{}.{}", name, method);
                         self.advance();
                    }
                }

                if self.current_token() == &Token::LeftParen {
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
                    return Expression::FunctionCall { target: name, args };
                }

                Expression::Variable(name)
            }
            _ => { self.advance(); Expression::LiteralStr("".to_string()) }
        }
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() { self.position += 1; }
    }

    fn current_token(&self) -> &Token {
        if self.position >= self.tokens.len() { return &Token::EOF; }
        &self.tokens[self.position]
    }
    
    fn peek(&self) -> &Token {
        if self.position + 1 >= self.tokens.len() { return &Token::EOF; }
        &self.tokens[self.position + 1]
    }
}