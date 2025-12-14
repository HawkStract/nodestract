use crate::ast::{Program, Statement, Expression};
use crate::lexer::Token;
use super::Parser;

impl Parser {
    pub fn parse(&mut self) -> Program {
        let mut statements = Vec::new();

        while self.position < self.tokens.len() {
            let token = self.current_token().clone();
            
            match token {
                Token::EOF => break,
                Token::Use => statements.push(self.parse_capability()),
                Token::Lock | Token::Stract | Token::Vault => {
                    let is_mut = matches!(token, Token::Stract);
                    let is_sec = matches!(token, Token::Vault);
                    statements.push(self.parse_var_decl(is_mut, is_sec));
                },
                Token::Func => statements.push(self.parse_function()),
                Token::Module => { self.advance(); self.advance(); },
                Token::Identifier(_) => {
                    if self.peek() == &Token::Equal {
                        statements.push(self.parse_assignment());
                    } else {
                        statements.push(self.parse_func_call_stmt());
                    }
                },
                Token::If => statements.push(self.parse_if_statement()),
                Token::While => statements.push(self.parse_while_statement()),
                Token::For => statements.push(self.parse_for_statement()),
                Token::Return => statements.push(self.parse_return_statement()),
                _ => self.advance(),
            }
        }

        Program { statements }
    }

    fn parse_capability(&mut self) -> Statement {
        self.advance();
        let service_name = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => "Unknown".to_string(),
        };
        self.advance(); 
        if self.current_token() == &Token::LeftBrace {
             self.advance(); 
             while self.current_token() != &Token::RightBrace && self.current_token() != &Token::EOF { 
                 self.advance(); 
             }
             self.advance(); 
        }
        Statement::CapabilityUse { service: service_name, params: vec![] }
    }

    fn parse_var_decl(&mut self, is_mutable: bool, is_secure: bool) -> Statement {
        self.advance();
        let name = match self.current_token() {
            Token::Identifier(s) => s.clone(), _ => "Unknown".to_string(),
        };
        self.advance(); self.advance(); 
        let value = self.parse_expression();
        Statement::VarDecl { is_mutable, is_secure, name, value }
    }
    
    fn parse_function(&mut self) -> Statement {
        self.advance();
        let name = match self.current_token() {
            Token::Identifier(s) => s.clone(), _ => "Anon".to_string(),
        };
        self.advance(); self.advance(); 
        let mut params = Vec::new();
        if self.current_token() != &Token::RightParen {
            if let Token::Identifier(p) = self.current_token() { params.push(p.clone()); self.advance(); }
            while self.current_token() == &Token::Comma {
                self.advance();
                if let Token::Identifier(p) = self.current_token() { params.push(p.clone()); self.advance(); }
            }
        }
        self.advance(); self.advance(); 
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
                },
                Token::Return => body.push(self.parse_return_statement()),
                Token::If => body.push(self.parse_if_statement()),
                Token::While => body.push(self.parse_while_statement()),
                Token::For => body.push(self.parse_for_statement()),
                Token::Identifier(_) => {
                    if self.peek() == &Token::Equal { body.push(self.parse_assignment()); } 
                    else { body.push(self.parse_func_call_stmt()); }
                },
                _ => self.advance(),
            }
        }
        self.advance(); 
        body
    }
    
    fn parse_if_statement(&mut self) -> Statement {
        self.advance(); let condition = self.parse_expression();
        while self.current_token() != &Token::LeftBrace { self.advance(); }
        self.advance(); let then_branch = self.parse_block();
        let mut else_branch = None;
        if self.current_token() == &Token::Else {
            self.advance(); while self.current_token() != &Token::LeftBrace { self.advance(); }
            self.advance(); else_branch = Some(self.parse_block());
        }
        Statement::IfStatement { condition, then_branch, else_branch }
    }

    fn parse_while_statement(&mut self) -> Statement {
        self.advance(); let condition = self.parse_expression();
        while self.current_token() != &Token::LeftBrace { self.advance(); }
        self.advance(); let body = self.parse_block();
        Statement::WhileStatement { condition, body }
    }

    fn parse_for_statement(&mut self) -> Statement {
        self.advance(); // Consuma 'for'
        let iterator = match self.current_token() {
            Token::Identifier(s) => s.clone(), _ => "i".to_string()
        };
        self.advance(); // Consuma nome iteratore
        
        let start = self.parse_primary(); 
        // RIMOSSO: self.advance(); <--- QUESTO ERA IL BUG! parse_primary avanza già da solo.
        
        let end = self.parse_primary();
        
        while self.current_token() != &Token::LeftBrace { self.advance(); }
        self.advance(); 
        let body = self.parse_block();
        Statement::ForStatement { iterator, start, end, body }
    }

    fn parse_return_statement(&mut self) -> Statement {
        self.advance(); let value = self.parse_expression(); Statement::ReturnStatement { value }
    }

    fn parse_assignment(&mut self) -> Statement {
        let name = match self.current_token() {
            Token::Identifier(s) => s.clone(), _ => "Unknown".to_string(),
        };
        self.advance(); self.advance(); 
        let value = self.parse_expression();
        Statement::Assignment { name, value }
    }

    fn parse_func_call_stmt(&mut self) -> Statement {
        let mut target = match self.current_token() {
            Token::Identifier(s) => s.clone(), _ => "".to_string()
        };
        self.advance();
        if self.current_token() == &Token::Dot {
            self.advance();
            if let Token::Identifier(method) = self.current_token() {
                target = format!("{}.{}", target, method); self.advance();
            }
        }
        self.advance(); 
        let mut args = Vec::new();
        if self.current_token() != &Token::RightParen {
            args.push(self.parse_expression());
            while self.current_token() == &Token::Comma { self.advance(); args.push(self.parse_expression()); }
        }
        self.advance(); 
        Statement::Expr(Expression::FunctionCall { target, args })
    }
}