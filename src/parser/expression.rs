use crate::ast::{Expression};
use crate::lexer::Token;
use super::Parser;

impl Parser {
    pub fn parse_expression(&mut self) -> Expression {
        let mut left = self.parse_term();
        while matches!(self.current_token(), Token::Plus | Token::Minus | Token::EqualEqual | Token::Greater | Token::Less) {
            let operator = match self.current_token() {
                Token::Plus => "+".to_string(), Token::Minus => "-".to_string(),
                Token::EqualEqual => "==".to_string(), Token::Greater => ">".to_string(),
                Token::Less => "<".to_string(), _ => "".to_string(),
            };
            self.advance(); let right = self.parse_term();
            left = Expression::BinaryOp { left: Box::new(left), operator, right: Box::new(right) };
        }
        left
    }

    pub fn parse_term(&mut self) -> Expression {
        let mut left = self.parse_unary();
        while matches!(self.current_token(), Token::Star | Token::Slash) {
            let operator = match self.current_token() {
                Token::Star => "*".to_string(), Token::Slash => "/".to_string(), _ => "".to_string(),
            };
            self.advance(); let right = self.parse_unary();
            left = Expression::BinaryOp { left: Box::new(left), operator, right: Box::new(right) };
        }
        left
    }

    pub fn parse_unary(&mut self) -> Expression {
        if self.current_token() == &Token::Minus {
            self.advance(); let right = self.parse_unary();
            return Expression::BinaryOp {
                left: Box::new(Expression::LiteralNum(0.0)),
                operator: "-".to_string(),
                right: Box::new(right),
            };
        }
        self.parse_primary()
    }

    pub fn parse_primary(&mut self) -> Expression {
        match self.current_token() {
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
                        
                        if self.current_token() == &Token::Colon {
                            self.advance();
                        }
                        
                        let value = self.parse_expression();
                        pairs.push((key, value));
                        
                        if self.current_token() == &Token::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.advance();
                Expression::Map(pairs)
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
            _ => { self.advance(); Expression::LiteralStr("".to_string()) }
        }
    }
}