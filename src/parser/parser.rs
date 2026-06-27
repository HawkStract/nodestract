use crate::engine::lexer::Token;

pub mod expression;
pub mod statement;

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

    /// Primary parse function coordinating validation and AST building
    pub fn parse(&mut self) -> Result<crate::engine::ast::Program, String> {
        // 1. Run syntactical structure and delimiter checks
        self.pre_check()?;

        // 2. Parse token stream into AST statements
        self.position = 0;
        let mut statements = Vec::new();
        while self.position < self.tokens.len() && self.current_token() != &Token::EOF {
            statements.push(self.parse_statement()?);
        }

        Ok(crate::engine::ast::Program { statements })
    }

    /// Access the current token
    pub fn current_token(&self) -> &Token {
        if self.position >= self.tokens.len() {
            &Token::EOF
        } else {
            &self.tokens[self.position]
        }
    }

    /// Peeks at the next token
    pub fn peek(&self) -> &Token {
        if self.position + 1 >= self.tokens.len() {
            &Token::EOF
        } else {
            &self.tokens[self.position + 1]
        }
    }

    /// Advances the position pointer
    pub fn advance(&mut self) -> &Token {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        self.current_token()
    }

    /// Consumes a specific expected token or returns an error
    pub fn consume(&mut self, expected: &Token, err_msg: &str) -> Result<(), String> {
        if self.current_token() == expected {
            self.advance();
            Ok(())
        } else {
            Err(format!("Syntax Error: {} (found {:?}", err_msg, self.current_token()))
        }
    }

    /// Per-parse checks for brackets, variable naming collisions, and structural blocks
    fn pre_check(&self) -> Result<(), String> {
        // Validate that keywords are not used as variable or function names
        for i in 0..self.tokens.len() {
            if let Token::Keyword(ref kw) = self.tokens[i] {
                if kw == "let" || kw == "const" {
                    if i + 1 < self.tokens.len() {
                        if let Token::Keyword(ref name) = self.tokens[i + 1] {
                            return Err(format!("Cannot use keyword '{}' as a variable name", name));
                        }
                    }
                }
                if kw == "function" {
                    if i + 1 < self.tokens.len() {
                        if let Token::Keyword(ref name) = self.tokens[i + 1] {
                            return Err(format!("Cannot use keyword '{}' as a function name", name));
                        }
                    }
                }
            }
        }

        // Validate structural blocks: check for `Identifier ( ... ) {` without a preceding `function` keyword.
        for idx in 0..self.tokens.len() {
            if let Token::Delimiter(ref sym) = self.tokens[idx] {
                if sym == "{" {
                    if idx > 0 {
                        let mut search_idx = idx - 1;
                        if let Token::Delimiter(ref close_paren) = self.tokens[search_idx] {
                            if close_paren == ")" {
                                let mut paren_stack = 1;
                                while search_idx > 0 && paren_stack > 0 {
                                    search_idx -= 1;
                                    if let Token::Delimiter(ref p) = self.tokens[search_idx] {
                                        if p == ")" {
                                            paren_stack += 1;
                                        } else if p == "(" {
                                            paren_stack -= 1;
                                        }
                                    }
                                }
                                if paren_stack == 0 && search_idx > 0 {
                                    search_idx -= 1;
                                    if let Token::Identifier(_) = &self.tokens[search_idx] {
                                        let mut is_function = false;
                                        if search_idx > 0 {
                                            if let Token::Keyword(ref kw) = &self.tokens[search_idx - 1] {
                                                if kw == "function" {
                                                    is_function = true;
                                                }
                                            }
                                        }
                                        if !is_function {
                                            return Err("Unexpected block '{' following expression or function call".to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Delimiter balancing check (brackets, braces, parentheses)
        let mut stack = Vec::new();
        for (idx, token) in self.tokens.iter().enumerate() {
            if let Token::Delimiter(ref sym) = token {
                match sym.as_str() {
                    "{" | "(" | "[" => {
                        stack.push((sym.as_str(), idx));
                    }
                    "}" => {
                        match stack.pop() {
                            Some(("{", _)) => {}
                            Some((expected, start_idx)) => {
                                return Err(format!(
                                    "Mismatched closing brace '}}'. Expected closing for '{}' opened at token position {}",
                                    expected, start_idx
                                ));
                            }
                            None => {
                                return Err("Unmatched closing brace '}'".to_string());
                            }
                        }
                    }
                    ")" => {
                        match stack.pop() {
                            Some(("(", _)) => {}
                            Some((expected, start_idx)) => {
                                return Err(format!(
                                    "Mismatched closing parenthesis ')'. Expected closing for '{}' opened at token position {}",
                                    expected, start_idx
                                ));
                            }
                            None => {
                                return Err("Unmatched closing parenthesis ')'".to_string());
                            }
                        }
                    }
                    "]" => {
                        match stack.pop() {
                            Some(("[", _)) => {}
                            Some((expected, start_idx)) => {
                                return Err(format!(
                                    "Mismatched closing bracket ']'. Expected closing for '{}' opened at token position {}",
                                    expected, start_idx
                                ));
                            }
                            None => {
                                return Err("Unmatched closing bracket ']'".to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some((unclosed, idx)) = stack.pop() {
            return Err(format!(
                "Unclosed delimiter '{}' opened at token position {}",
                unclosed, idx
            ));
        }

        Ok(())
    }
}