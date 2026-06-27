use crate::engine::ast::Expression;
use crate::engine::lexer::Token;
use super::Parser;

impl Parser {
    pub fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_logical_or()?;
        if self.current_token() == &Token::Operator("?".to_string()) {
            self.advance();
            let true_expr = self.parse_expression()?;
            self.consume(&Token::Delimiter(":".to_string()), "Expected ':' in ternary expression")?;
            let false_expr = self.parse_expression()?;
            expr = Expression::Ternary {
                condition: Box::new(expr),
                true_expr: Box::new(true_expr),
                false_expr: Box::new(false_expr),
            };
        }
        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_logical_and()?;
        while self.current_token() == &Token::Operator("||".to_string()) {
            self.advance();
            let right = self.parse_logical_and()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: "||".to_string(),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_equality()?;
        while self.current_token() == &Token::Operator("&&".to_string()) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: "&&".to_string(),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison()?;
        while let Token::Operator(ref op) = self.current_token() {
            if op == "==" || op == "!=" {
                let operator = op.clone();
                self.advance();
                let right = self.parse_comparison()?;
                left = Expression::BinaryOp {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_term()?;
        while let Token::Operator(ref op) = self.current_token() {
            if op == ">" || op == ">=" || op == "<" || op == "<=" {
                let operator = op.clone();
                self.advance();
                let right = self.parse_term()?;
                left = Expression::BinaryOp {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    pub fn parse_term(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_factor()?;
        while let Token::Operator(ref op) = self.current_token() {
            if op == "+" || op == "-" {
                let operator = op.clone();
                self.advance();
                let right = self.parse_factor()?;
                left = Expression::BinaryOp {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    pub fn parse_factor(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_unary()?;
        while let Token::Operator(ref op) = self.current_token() {
            if op == "*" || op == "/" {
                let operator = op.clone();
                self.advance();
                let right = self.parse_unary()?;
                left = Expression::BinaryOp {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    pub fn parse_unary(&mut self) -> Result<Expression, String> {
        if let Token::Operator(ref op) = self.current_token() {
            if op == "-" || op == "!" {
                let operator = op.clone();
                self.advance();
                let operand = self.parse_unary()?;
                return Ok(Expression::UnaryOp {
                    operator,
                    operand: Box::new(operand),
                });
            }
        }
        self.parse_primary()
    }

    pub fn parse_primary(&mut self) -> Result<Expression, String> {
        let token = self.current_token().clone();
        match token {
            Token::Keyword(ref kw) => {
                if kw == "true" {
                    self.advance();
                    Ok(Expression::LiteralBool(true))
                } else if kw == "false" {
                    self.advance();
                    Ok(Expression::LiteralBool(false))
                } else if kw == "null" {
                    self.advance();
                    Ok(Expression::LiteralNull)
                } else if kw == "await" {
                    self.advance();
                    let value = self.parse_unary()?;
                    Ok(Expression::Await(Box::new(value)))
                } else {
                    // Try to parse as a built-in function or variable keyword (e.g. print, sin, etc.)
                    self.parse_identifier_or_keyword_expr(kw.clone())
                }
            }
            Token::Delimiter(ref sym) if sym == "[" => {
                self.advance(); // consume [
                let mut elements = Vec::new();
                if self.current_token() != &Token::Delimiter("]".to_string()) {
                    elements.push(self.parse_expression()?);
                    while self.current_token() == &Token::Delimiter(",".to_string()) {
                        self.advance();
                        elements.push(self.parse_expression()?);
                    }
                }
                self.consume(&Token::Delimiter("]".to_string()), "Expected ']' at end of array")?;
                Ok(Expression::Array(elements))
            }
            Token::Delimiter(ref sym) if sym == "{" => {
                self.advance(); // consume {
                let mut pairs = Vec::new();
                if self.current_token() != &Token::Delimiter("}".to_string()) {
                    loop {
                        let key = match self.current_token() {
                            Token::StringLiteral(s) | Token::Identifier(s) => s.clone(),
                            Token::Keyword(k) => k.clone(),
                            _ => return Err("Expected string or identifier as key in map literal".to_string()),
                        };
                        self.advance();
                        self.consume(&Token::Delimiter(":".to_string()), "Expected ':' after map key")?;
                        let value = self.parse_expression()?;
                        pairs.push((key, value));
                        if self.current_token() == &Token::Delimiter(",".to_string()) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.consume(&Token::Delimiter("}".to_string()), "Expected '}' at end of map literal")?;
                Ok(Expression::Map(pairs))
            }
            Token::StringLiteral(s) => {
                self.advance();
                Ok(Expression::LiteralStr(s))
            }
            Token::Number(n) => {
                self.advance();
                Ok(Expression::LiteralNum(n))
            }
            Token::Identifier(s) => {
                self.parse_identifier_or_keyword_expr(s)
            }
            Token::Delimiter(ref sym) if sym == "(" => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(&Token::Delimiter(")".to_string()), "Expected ')' after parenthesized expression")?;
                Ok(expr)
            }
            _ => Err(format!(
                "Syntax Error: Unexpected token {:?} at start of expression",
                self.current_token()
            )),
        }
    }

    fn parse_identifier_or_keyword_expr(&mut self, name_str: String) -> Result<Expression, String> {
        let mut name = name_str;
        self.advance();

        // Member access (dot notation)
        if self.current_token() == &Token::Delimiter(".".to_string()) {
            self.advance();
            if let Token::Identifier(ref method) = self.current_token() {
                name = format!("{}.{}", name, method);
                self.advance();
            }
        }

        let mut expr = Expression::Variable(name.clone());
        loop {
            if self.current_token() == &Token::Delimiter("[".to_string()) {
                self.advance();
                let index = self.parse_expression()?;
                self.consume(&Token::Delimiter("]".to_string()), "Expected ']' after index")?;
                expr = Expression::Index {
                    target: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.current_token() == &Token::Delimiter("(".to_string()) {
                self.advance();
                let mut args = Vec::new();
                if self.current_token() != &Token::Delimiter(")".to_string()) {
                    args.push(self.parse_expression()?);
                    while self.current_token() == &Token::Delimiter(",".to_string()) {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                }
                self.consume(&Token::Delimiter(")".to_string()), "Expected ')' after arguments")?;
                expr = Expression::FunctionCall {
                    target: name.clone(),
                    args,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }
}