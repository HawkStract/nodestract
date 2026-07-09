use crate::engine::lexer::Token;

pub mod expression;
pub mod statement;

pub struct Parser {
    pub tokens: Vec<Token>,
    pub position: usize,
    pub loop_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            loop_depth: 0,
        }
    }

    /// Funzione di parsing principale.
    pub fn parse(&mut self) -> Result<crate::engine::ast::Program, String> {
        // 1. Controlli strutturali preliminari sui delimitatori
        self.pre_check()?;

        // 2. Converte i token in statement dell'AST
        self.position = 0;
        let mut statements = Vec::new();
        while self.position < self.tokens.len() && self.current_token() != &Token::EOF {
            statements.push(self.parse_statement()?);
        }

        Ok(crate::engine::ast::Program { statements })
    }

    /// Restituisce il token corrente.
    pub fn current_token(&self) -> &Token {
        if self.position >= self.tokens.len() {
            &Token::EOF
        } else {
            &self.tokens[self.position]
        }
    }

    /// Guarda il token successivo.
    pub fn peek(&self) -> &Token {
        if self.position + 1 >= self.tokens.len() {
            &Token::EOF
        } else {
            &self.tokens[self.position + 1]
        }
    }

    /// Avanza l'indice del parser.
    pub fn advance(&mut self) -> &Token {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        self.current_token()
    }

    /// Consuma un token specifico o solleva un errore sintattico.
    pub fn consume(&mut self, expected: &Token, err_msg: &str) -> Result<(), String> {
        if self.current_token() == expected {
            self.advance();
            Ok(())
        } else {
            Err(format!("Syntax Error: {} (found {:?}", err_msg, self.current_token()))
        }
    }

    /// Controlli pre-parsing per bilanciamento parentesi e collisioni di nomi.
    fn pre_check(&self) -> Result<(), String> {
        // Impedisce l'uso di keyword come nomi di variabili/funzioni
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

        // Evita blocchi graffiati spuri legati a chiamate o espressioni
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

        // Controllo del bilanciamento di parentesi tonde, quadre e graffe
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

#[cfg(test)]
mod tests {
    use crate::engine::Engine;

    #[test]
    fn test_invalid_break_and_continue() {
        let engine = Engine::new();
        
        // break outside of loop
        let source_break = "importa italiano da translate\ninterrompi\n";
        // Let's just run it via Engine. Engine should print/handle errors. But we can test validate_imports + tokenization + parser directly.
        let (stripped, import_mgr) = crate::engine::check::validate_imports(source_break, &engine.translation_engine).unwrap();
        let filtered = crate::engine::filter::FilteredEngine::new(&engine.translation_engine, &import_mgr);
        let mut lexer = crate::engine::lexer::Lexer::new(&stripped);
        let tokens = lexer.tokenize(&engine.translation_engine, &filtered);
        let mut parser = crate::engine::parser::Parser::new(tokens);
        let res = parser.parse();
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("only allowed inside loops"));

        // continue outside of loop
        let source_continue = "importa italiano da translate\ncontinua\n";
        let (stripped, import_mgr) = crate::engine::check::validate_imports(source_continue, &engine.translation_engine).unwrap();
        let filtered = crate::engine::filter::FilteredEngine::new(&engine.translation_engine, &import_mgr);
        let mut lexer = crate::engine::lexer::Lexer::new(&stripped);
        let tokens = lexer.tokenize(&engine.translation_engine, &filtered);
        let mut parser = crate::engine::parser::Parser::new(tokens);
        let res = parser.parse();
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("only allowed inside loops"));

        // break inside function inside loop (illegal)
        let source_func = "importa italiano da translate\nmentre (vero) {\nfunzione f() {\ninterrompi\n}\n}\n";
        let (stripped, import_mgr) = crate::engine::check::validate_imports(source_func, &engine.translation_engine).unwrap();
        let filtered = crate::engine::filter::FilteredEngine::new(&engine.translation_engine, &import_mgr);
        let mut lexer = crate::engine::lexer::Lexer::new(&stripped);
        let tokens = lexer.tokenize(&engine.translation_engine, &filtered);
        let mut parser = crate::engine::parser::Parser::new(tokens);
        let res = parser.parse();
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("only allowed inside loops"));
    }
}