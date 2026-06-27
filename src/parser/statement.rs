use crate::engine::ast::{Statement, Expression};
use crate::engine::lexer::Token;
use super::Parser;

impl Parser {
    /// Parses a single statement from the token stream
    pub fn parse_statement(&mut self) -> Result<Statement, String> {
        let token = self.current_token().clone();
        match token {
            Token::Keyword(ref kw) => {
                match kw.as_str() {
                    "let" | "const" => {
                        let is_mut = kw == "let";
                        self.parse_var_decl(is_mut)
                    }
                    "if" => self.parse_if_statement(),
                    "while" => self.parse_while_statement(),
                    "for" => self.parse_for_statement(),
                    "switch" => self.parse_switch_statement(),
                    "try" => self.parse_try_catch_statement(),
                    "throw" => self.parse_throw_statement(),
                    "break" => {
                        self.advance();
                        Ok(Statement::Break)
                    }
                    "continue" => {
                        self.advance();
                        Ok(Statement::Continue)
                    }
                    "return" => self.parse_return_statement(),
                    "function" => self.parse_function(false),
                    "async" => {
                        self.advance(); // consume async
                        self.consume(
                            &Token::Keyword("function".to_string()),
                            "Expected 'function' keyword after 'async'",
                        )?;
                        self.parse_function(true)
                    }
                    _ => {
                        let expr = self.parse_expression()?;
                        Ok(Statement::Expr(expr))
                    }
                }
            }
            Token::Identifier(_) => self.parse_identifier_statement(),
            Token::EOF => Err("Syntax Error: Unexpected End Of File".to_string()),
            _ => {
                let expr = self.parse_expression()?;
                Ok(Statement::Expr(expr))
            }
        }
    }

    fn parse_var_decl(&mut self, is_mutable: bool) -> Result<Statement, String> {
        self.advance(); // consume let/const
        let name = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => return Err("Expected identifier for variable name".to_string()),
        };
        self.advance();
        self.consume(&Token::Operator("=".to_string()), "Expected '=' after variable name")?;
        let value = self.parse_expression()?;
        Ok(Statement::VarDecl { is_mutable, name, value })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume if
        let has_paren = self.current_token() == &Token::Delimiter("(".to_string());
        if has_paren {
            self.advance();
        }
        let condition = self.parse_expression()?;
        if has_paren {
            self.consume(&Token::Delimiter(")".to_string()), "Expected ')' after condition")?;
        }
        self.consume(&Token::Delimiter("{".to_string()), "Expected '{' before then branch")?;
        let then_branch = self.parse_block()?;
        let mut else_branch = None;
        if self.current_token() == &Token::Keyword("else".to_string()) {
            self.advance(); // consume else
            if self.current_token() == &Token::Keyword("if".to_string()) {
                else_branch = Some(vec![self.parse_if_statement()?]);
            } else {
                self.consume(&Token::Delimiter("{".to_string()), "Expected '{' before else branch")?;
                else_branch = Some(self.parse_block()?);
            }
        }
        Ok(Statement::IfStatement { condition, then_branch, else_branch })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume while
        let has_paren = self.current_token() == &Token::Delimiter("(".to_string());
        if has_paren {
            self.advance();
        }
        let condition = self.parse_expression()?;
        if has_paren {
            self.consume(&Token::Delimiter(")".to_string()), "Expected ')' after condition")?;
        }
        self.consume(&Token::Delimiter("{".to_string()), "Expected '{' before while body")?;
        let body = self.parse_block()?;
        Ok(Statement::WhileStatement { condition, body })
    }

    fn parse_for_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume for
        let has_paren = self.current_token() == &Token::Delimiter("(".to_string());
        if has_paren {
            self.advance();
        }

        let iterator = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => return Err("Expected identifier in for loop iterator".to_string()),
        };
        self.advance();

        self.consume(&Token::Keyword("in".to_string()), "Expected 'in' keyword in for loop")?;
        let start = self.parse_expression()?;

        // Support optional range operator like '..'
        let mut end = start.clone();
        if let Token::Operator(ref op) = self.current_token() {
            if op == ".." {
                self.advance();
                end = self.parse_expression()?;
            }
        }

        if has_paren {
            self.consume(&Token::Delimiter(")".to_string()), "Expected ')' after for loop range")?;
        }
        self.consume(&Token::Delimiter("{".to_string()), "Expected '{' before for body")?;
        let body = self.parse_block()?;
        Ok(Statement::ForStatement { iterator, start, end, body })
    }

    fn parse_switch_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume switch
        let has_paren = self.current_token() == &Token::Delimiter("(".to_string());
        if has_paren {
            self.advance();
        }
        let discriminant = self.parse_expression()?;
        if has_paren {
            self.consume(&Token::Delimiter(")".to_string()), "Expected ')' after switch value")?;
        }
        self.consume(&Token::Delimiter("{".to_string()), "Expected '{' before switch block")?;

        let mut cases = Vec::new();
        let mut default_case = None;

        while self.current_token() != &Token::Delimiter("}".to_string()) && self.current_token() != &Token::EOF {
            match self.current_token() {
                Token::Keyword(ref kw) if kw == "case" => {
                    self.advance();
                    let test = self.parse_expression()?;
                    self.consume(&Token::Delimiter(":".to_string()), "Expected ':' after case value")?;
                    let mut body = Vec::new();
                    while self.current_token() != &Token::Keyword("case".to_string())
                        && self.current_token() != &Token::Keyword("default".to_string())
                        && self.current_token() != &Token::Delimiter("}".to_string())
                        && self.current_token() != &Token::EOF
                    {
                        body.push(self.parse_statement()?);
                    }
                    cases.push((test, body));
                }
                Token::Keyword(ref kw) if kw == "default" => {
                    self.advance();
                    self.consume(&Token::Delimiter(":".to_string()), "Expected ':' after default")?;
                    let mut body = Vec::new();
                    while self.current_token() != &Token::Keyword("case".to_string())
                        && self.current_token() != &Token::Keyword("default".to_string())
                        && self.current_token() != &Token::Delimiter("}".to_string())
                        && self.current_token() != &Token::EOF
                    {
                        body.push(self.parse_statement()?);
                    }
                    default_case = Some(body);
                }
                _ => {
                    return Err(format!(
                        "Expected 'case' or 'default' inside switch block, found {:?}",
                        self.current_token()
                    ));
                }
            }
        }
        self.consume(&Token::Delimiter("}".to_string()), "Expected '}' at end of switch block")?;
        Ok(Statement::SwitchStatement { discriminant, cases, default_case })
    }

    fn parse_try_catch_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume try
        self.consume(&Token::Delimiter("{".to_string()), "Expected '{' after 'try'")?;
        let try_block = self.parse_block()?;

        let mut catch_variable = None;
        let mut catch_block = Vec::new();

        if self.current_token() == &Token::Keyword("catch".to_string()) {
            self.advance(); // consume catch
            let has_var_paren = self.current_token() == &Token::Delimiter("(".to_string());
            if has_var_paren {
                self.advance();
                if let Token::Identifier(ref var) = self.current_token() {
                    catch_variable = Some(var.clone());
                    self.advance();
                }
                self.consume(&Token::Delimiter(")".to_string()), "Expected ')' after catch variable")?;
            }
            self.consume(&Token::Delimiter("{".to_string()), "Expected '{' after 'catch'")?;
            catch_block = self.parse_block()?;
        }

        let mut finally_block = None;
        if self.current_token() == &Token::Keyword("finally".to_string()) {
            self.advance(); // consume finally
            self.consume(&Token::Delimiter("{".to_string()), "Expected '{' after 'finally'")?;
            finally_block = Some(self.parse_block()?);
        }

        Ok(Statement::TryCatchStatement { try_block, catch_variable, catch_block, finally_block })
    }

    fn parse_throw_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume throw
        let value = self.parse_expression()?;
        Ok(Statement::ThrowStatement { value })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume return
        let value = self.parse_expression()?;
        Ok(Statement::ReturnStatement { value })
    }

    fn parse_function(&mut self, is_async: bool) -> Result<Statement, String> {
        self.advance(); // consume function
        let name = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => return Err("Expected identifier for function name".to_string()),
        };
        self.advance();
        self.consume(&Token::Delimiter("(".to_string()), "Expected '(' after function name")?;
        let mut params = Vec::new();
        if self.current_token() != &Token::Delimiter(")".to_string()) {
            if let Token::Identifier(ref p) = self.current_token() {
                params.push(p.clone());
                self.advance();
            }
            while self.current_token() == &Token::Delimiter(",".to_string()) {
                self.advance();
                if let Token::Identifier(ref p) = self.current_token() {
                    params.push(p.clone());
                    self.advance();
                }
            }
        }
        self.consume(&Token::Delimiter(")".to_string()), "Expected ')' after parameters")?;
        self.consume(&Token::Delimiter("{".to_string()), "Expected '{' before function body")?;
        let body = self.parse_block()?;
        Ok(Statement::FunctionDecl { is_async, name, params, body })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, String> {
        let mut body = Vec::new();
        while self.current_token() != &Token::Delimiter("}".to_string()) && self.current_token() != &Token::EOF {
            body.push(self.parse_statement()?);
        }
        self.consume(&Token::Delimiter("}".to_string()), "Expected '}' at end of block")?;
        Ok(body)
    }

    fn parse_identifier_statement(&mut self) -> Result<Statement, String> {
        let name = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };

        let next = self.peek().clone();
        match next {
            Token::Operator(ref op) if op == "=" => {
                self.advance(); // consume name
                self.advance(); // consume =
                let value = self.parse_expression()?;
                Ok(Statement::Assignment { name, value })
            }
            Token::Operator(ref op) if op == "++" || op == "--" => {
                let actual_op = if op == "++" { "+" } else { "-" };
                self.advance(); // consume name
                self.advance(); // consume ++/--
                Ok(Statement::Assignment {
                    name: name.clone(),
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::Variable(name)),
                        operator: actual_op.to_string(),
                        right: Box::new(Expression::LiteralNum(1.0)),
                    },
                })
            }
            Token::Operator(ref op) if op == "+=" || op == "-=" || op == "*=" || op == "/=" => {
                let actual_op = &op[0..1];
                self.advance(); // consume name
                self.advance(); // consume op
                let right = self.parse_expression()?;
                Ok(Statement::Assignment {
                    name: name.clone(),
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::Variable(name)),
                        operator: actual_op.to_string(),
                        right: Box::new(right),
                    },
                })
            }
            _ => {
                let expr = self.parse_expression()?;
                Ok(Statement::Expr(expr))
            }
        }
    }
}