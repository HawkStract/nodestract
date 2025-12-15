use crate::ast::{Expression};
use crate::lexer::Token;
use super::Parser;

impl Parser {
    pub fn parse_expression(&mut self) -> Expression {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Expression {
        let mut left = self.parse_logical_and();
        while self.current_token() == &Token::PipePipe {
            self.advance();
            let right = self.parse_logical_and();
            left = Expression::BinaryOp { left: Box::new(left), operator: "||".to_string(), right: Box::new(right) };
        }
        left
    }

    fn parse_logical_and(&mut self) -> Expression {
        let mut left = self.parse_equality();
        while self.current_token() == &Token::AmperAmper {
            self.advance();
            let right = self.parse_equality();
            left = Expression::BinaryOp { left: Box::new(left), operator: "&&".to_string(), right: Box::new(right) };
        }
        left
    }

    fn parse_equality(&mut self) -> Expression {
        let mut left = self.parse_comparison();
        while matches!(self.current_token(), Token::EqualEqual | Token::BangEqual) {
            let operator = match self.current_token() {
                Token::EqualEqual => "==".to_string(),
                Token::BangEqual => "!=".to_string(),
                _ => "".to_string(),
            };
            self.advance();
            let right = self.parse_comparison();
            left = Expression::BinaryOp { left: Box::new(left), operator, right: Box::new(right) };
        }
        left
    }

    fn parse_comparison(&mut self) -> Expression {
        let mut left = self.parse_term();
        while matches!(self.current_token(), Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual) {
            let operator = match self.current_token() {
                Token::Greater => ">".to_string(),
                Token::GreaterEqual => ">=".to_string(),
                Token::Less => "<".to_string(),
                Token::LessEqual => "<=".to_string(),
                _ => "".to_string(),
            };
            self.advance();
            let right = self.parse_term();
            left = Expression::BinaryOp { left: Box::new(left), operator, right: Box::new(right) };
        }
        left
    }

    pub fn parse_term(&mut self) -> Expression {
        let mut left = self.parse_factor();
        while matches!(self.current_token(), Token::Plus | Token::Minus) {
            let operator = match self.current_token() {
                Token::Plus => "+".to_string(), Token::Minus => "-".to_string(), _ => "".to_string(),
            };
            self.advance(); 
            let right = self.parse_factor();
            left = Expression::BinaryOp { left: Box::new(left), operator, right: Box::new(right) };
        }
        left
    }

    pub fn parse_factor(&mut self) -> Expression {
        let mut left = self.parse_unary();
        while matches!(self.current_token(), Token::Star | Token::Slash) {
            let operator = match self.current_token() {
                Token::Star => "*".to_string(), Token::Slash => "/".to_string(), _ => "".to_string(),
            };
            self.advance(); 
            let right = self.parse_unary();
            left = Expression::BinaryOp { left: Box::new(left), operator, right: Box::new(right) };
        }
        left
    }

    pub fn parse_unary(&mut self) -> Expression {
        // FIX: Supporto per '-' (negazione numerica) e '!' (negazione logica)
        if matches!(self.current_token(), Token::Minus | Token::Bang) {
            let operator = match self.current_token() {
                Token::Minus => "-".to_string(),
                Token::Bang => "!".to_string(),
                _ => "".to_string(),
            };
            self.advance(); 
            let operand = self.parse_unary(); // Ricorsivo per gestire !!true o --5
            return Expression::UnaryOp {
                operator,
                operand: Box::new(operand),
            };
        }
        self.parse_primary()
    }

    pub fn parse_primary(&mut self) -> Expression {
        match self.current_token() {
            Token::True => { self.advance(); Expression::LiteralBool(true) }
            Token::False => { self.advance(); Expression::LiteralBool(false) }
            Token::LeftBracket => {
                self.advance(); let mut elements = Vec::new();
                if self.current_token() != &Token::RightBracket {
                    elements.push(self.parse_expression());
                    while self.current_token() == &Token::Comma { self.advance(); elements.push(self.parse_expression()); }
                }
                self.advance(); Expression::Array(elements)
            }
            Token::LeftBrace => {
                self.advance();
                let mut pairs = Vec::new();
                if self.current_token() != &Token::RightBrace {
                    loop {
                        let key = match self.current_token() {
                            Token::StringLiteral(s) | Token::Identifier(s) => s.clone(),
                            _ => "Unknown".to_string(),
                        };
                        self.advance();
                        if self.current_token() == &Token::Colon { self.advance(); }
                        let value = self.parse_expression();
                        pairs.push((key, value));
                        if self.current_token() == &Token::Comma { self.advance(); } else { break; }
                    }
                }
                self.advance(); Expression::Map(pairs)
            }
            Token::StringLiteral(s) => { let val = s.clone(); self.advance(); Expression::LiteralStr(val) }
            Token::Number(n) => { let val = *n; self.advance(); Expression::LiteralNum(val) }
            Token::Identifier(s) => {
                let mut name = s.clone(); self.advance();
                if self.current_token() == &Token::Dot {
                    self.advance();
                    if let Token::Identifier(method) = self.current_token() {
                         name = format!("{}.{}", name, method); self.advance();
                    }
                }
                let mut expr = Expression::Variable(name.clone());
                loop {
                    if self.current_token() == &Token::LeftBracket {
                        self.advance(); let index = self.parse_expression(); self.advance();
                        expr = Expression::Index { target: Box::new(expr), index: Box::new(index) };
                    } else if self.current_token() == &Token::LeftParen {
                        self.advance(); let mut args = Vec::new();
                        if self.current_token() != &Token::RightParen {
                            args.push(self.parse_expression());
                            while self.current_token() == &Token::Comma { self.advance(); args.push(self.parse_expression()); }
                        }
                        self.advance(); expr = Expression::FunctionCall { target: name.clone(), args };
                    } else { break; }
                }
                expr
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression();
                if self.current_token() == &Token::RightParen { self.advance(); }
                expr
            }
            _ => { self.advance(); Expression::LiteralStr("".to_string()) }
        }
    }
}