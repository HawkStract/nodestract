use crate::engine::lexer::{Token, TokenWithSpan};

pub mod expression;
pub mod statement;

pub struct Parser {
    pub tokens: Vec<TokenWithSpan>,
    pub position: usize,
    pub loop_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenWithSpan>) -> Self {
        Self {
            tokens,
            position: 0,
            loop_depth: 0,
        }
    }

    /// Funzione di parsing principale.
    pub fn parse(&mut self, translation_engine: &crate::engine::translate::TranslationEngine, import_manager: &crate::engine::import::ImportManager) -> Result<crate::engine::ast::Program, String> {
        // 1. Controlli strutturali e di importazione preliminari
        self.pre_check(translation_engine, import_manager)?;

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
            &self.tokens[self.position].token
        }
    }

    pub fn current_location(&self) -> (usize, usize) {
        if self.position >= self.tokens.len() {
            if let Some(last) = self.tokens.last() {
                (last.line, last.col)
            } else {
                (1, 1)
            }
        } else {
            (self.tokens[self.position].line, self.tokens[self.position].col)
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
            let (line, col) = self.current_location();
            Err(format!(
                "Syntax Error (Line {}, Col {}): {} (found {:?})",
                line, col, err_msg, self.current_token()
            ))
        }
    }

    /// Controlli pre-parsing per bilanciamento parentesi e collisioni di nomi.
    fn pre_check(&self, translation_engine: &crate::engine::translate::TranslationEngine, import_manager: &crate::engine::import::ImportManager) -> Result<(), String> {
        // Rileva stringhe letterali non chiuse a fine file
        for token_ws in &self.tokens {
            if let Token::Unknown('"') = token_ws.token {
                return Err(format!(
                    "Syntax Error (Line {}, Col {}): Unclosed string literal",
                    token_ws.line, token_ws.col
                ));
            }
        }

        // Verifica che le funzioni di sistema usate siano effettivamente importate
        for token_ws in &self.tokens {
            if let Token::Identifier(ref name) = token_ws.token {
                if let Some((canonical, module)) = translation_engine.get_builtin_info(name) {
                    if !import_manager.is_member_active(canonical, module) {
                        return Err(format!(
                            "Import Error (Line {}, Col {}): Built-in function '{}' used but its library module '{}' was not imported",
                            token_ws.line, token_ws.col, name, module
                        ));
                    }
                }
            }
        }

        // Impedisce l'uso di keyword come nomi di variabili/funzioni
        for i in 0..self.tokens.len() {
            if let Token::Keyword(ref kw) = self.tokens[i].token {
                if kw == "let" || kw == "const" {
                    if i + 1 < self.tokens.len() {
                        if let Token::Keyword(ref name) = self.tokens[i + 1].token {
                            let line = self.tokens[i + 1].line;
                            let col = self.tokens[i + 1].col;
                            return Err(format!(
                                "Syntax Error (Line {}, Col {}): Cannot use keyword '{}' as a variable name",
                                line, col, name
                            ));
                        }
                    }
                }
                if kw == "function" {
                    if i + 1 < self.tokens.len() {
                        if let Token::Keyword(ref name) = self.tokens[i + 1].token {
                            let line = self.tokens[i + 1].line;
                            let col = self.tokens[i + 1].col;
                            return Err(format!(
                                "Syntax Error (Line {}, Col {}): Cannot use keyword '{}' as a function name",
                                line, col, name
                            ));
                        }
                    }
                }
            }
        }

        // Evita blocchi graffiati spuri legati a chiamate o espressioni
        for idx in 0..self.tokens.len() {
            if let Token::Delimiter(ref sym) = self.tokens[idx].token {
                if sym == "{" {
                    if idx > 0 {
                        let mut search_idx = idx - 1;
                        if let Token::Delimiter(ref close_paren) = self.tokens[search_idx].token {
                            if close_paren == ")" {
                                let mut paren_stack = 1;
                                while search_idx > 0 && paren_stack > 0 {
                                    search_idx -= 1;
                                    if let Token::Delimiter(ref p) = self.tokens[search_idx].token {
                                        if p == ")" {
                                            paren_stack += 1;
                                        } else if p == "(" {
                                            paren_stack -= 1;
                                        }
                                    }
                                }
                                if paren_stack == 0 && search_idx > 0 {
                                    search_idx -= 1;
                                    if let Token::Identifier(_) = &self.tokens[search_idx].token {
                                        let mut is_function = false;
                                        if search_idx > 0 {
                                            if let Token::Keyword(ref kw) = &self.tokens[search_idx - 1].token {
                                                if kw == "function" {
                                                    is_function = true;
                                                }
                                            }
                                        }
                                        if !is_function {
                                            let line = self.tokens[idx].line;
                                            let col = self.tokens[idx].col;
                                            return Err(format!(
                                                "Syntax Error (Line {}, Col {}): Unexpected block '{{' following expression or function call",
                                                line, col
                                            ));
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
        for (idx, token_ws) in self.tokens.iter().enumerate() {
            if let Token::Delimiter(ref sym) = token_ws.token {
                match sym.as_str() {
                    "{" | "(" | "[" => {
                        stack.push((sym.as_str(), idx));
                    }
                    "}" => {
                        match stack.pop() {
                            Some(("{", _)) => {}
                            Some((expected, start_idx)) => {
                                let line = token_ws.line;
                                let col = token_ws.col;
                                let opened_line = self.tokens[start_idx].line;
                                let opened_col = self.tokens[start_idx].col;
                                return Err(format!(
                                    "Syntax Error (Line {}, Col {}): Mismatched closing brace '}}'. Expected closing for '{}' opened at Line {}, Col {}",
                                    line, col, expected, opened_line, opened_col
                                ));
                            }
                            None => {
                                let line = token_ws.line;
                                let col = token_ws.col;
                                return Err(format!(
                                    "Syntax Error (Line {}, Col {}): Unmatched closing brace '}}'",
                                    line, col
                                ));
                            }
                        }
                    }
                    ")" => {
                        match stack.pop() {
                            Some(("(", _)) => {}
                            Some((expected, start_idx)) => {
                                let line = token_ws.line;
                                let col = token_ws.col;
                                let opened_line = self.tokens[start_idx].line;
                                let opened_col = self.tokens[start_idx].col;
                                return Err(format!(
                                    "Syntax Error (Line {}, Col {}): Mismatched closing parenthesis ')'. Expected closing for '{}' opened at Line {}, Col {}",
                                    line, col, expected, opened_line, opened_col
                                ));
                            }
                            None => {
                                let line = token_ws.line;
                                let col = token_ws.col;
                                return Err(format!(
                                    "Syntax Error (Line {}, Col {}): Unmatched closing parenthesis ')'",
                                    line, col
                                ));
                            }
                        }
                    }
                    "]" => {
                        match stack.pop() {
                            Some(("[", _)) => {}
                            Some((expected, start_idx)) => {
                                let line = token_ws.line;
                                let col = token_ws.col;
                                let opened_line = self.tokens[start_idx].line;
                                let opened_col = self.tokens[start_idx].col;
                                return Err(format!(
                                    "Syntax Error (Line {}, Col {}): Mismatched closing bracket ']'. Expected closing for '{}' opened at Line {}, Col {}",
                                    line, col, expected, opened_line, opened_col
                                ));
                            }
                            None => {
                                let line = token_ws.line;
                                let col = token_ws.col;
                                return Err(format!(
                                    "Syntax Error (Line {}, Col {}): Unmatched closing bracket ']'",
                                    line, col
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some((unclosed, idx)) = stack.pop() {
            let line = self.tokens[idx].line;
            let col = self.tokens[idx].col;
            return Err(format!(
                "Syntax Error (Line {}, Col {}): Unclosed delimiter '{}'",
                line, col, unclosed
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
        
        // interrompi fuori da un ciclo
        let source_break = "importa italiano da translate\ninterrompi\n";
        // Esegue il test validando gli import, la tokenizzazione ed il parsing
        let (stripped, import_mgr) = crate::engine::check::validate_imports(source_break, &engine.translation_engine).unwrap();
        let filtered = crate::engine::filter::FilteredEngine::new(&engine.translation_engine, &import_mgr);
        let mut lexer = crate::engine::lexer::Lexer::new(&stripped);
        let tokens = lexer.tokenize(&engine.translation_engine, &filtered);
        let mut parser = crate::engine::parser::Parser::new(tokens);
        let res = parser.parse(&engine.translation_engine, &import_mgr);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("only allowed inside loops"));

        // continua fuori da un ciclo
        let source_continue = "importa italiano da translate\ncontinua\n";
        let (stripped, import_mgr) = crate::engine::check::validate_imports(source_continue, &engine.translation_engine).unwrap();
        let filtered = crate::engine::filter::FilteredEngine::new(&engine.translation_engine, &import_mgr);
        let mut lexer = crate::engine::lexer::Lexer::new(&stripped);
        let tokens = lexer.tokenize(&engine.translation_engine, &filtered);
        let mut parser = crate::engine::parser::Parser::new(tokens);
        let res = parser.parse(&engine.translation_engine, &import_mgr);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("only allowed inside loops"));

        // interrompi dentro una funzione all'interno di un ciclo (illegale)
        let source_func = "importa italiano da translate\nmentre (vero) {\nfunzione f() {\ninterrompi\n}\n}\n";
        let (stripped, import_mgr) = crate::engine::check::validate_imports(source_func, &engine.translation_engine).unwrap();
        let filtered = crate::engine::filter::FilteredEngine::new(&engine.translation_engine, &import_mgr);
        let mut lexer = crate::engine::lexer::Lexer::new(&stripped);
        let tokens = lexer.tokenize(&engine.translation_engine, &filtered);
        let mut parser = crate::engine::parser::Parser::new(tokens);
        let res = parser.parse(&engine.translation_engine, &import_mgr);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("only allowed inside loops"));
    }

    #[test]
    fn test_short_circuit_and_try_finally_and_dot_notation() {
        let mut engine = Engine::new();
        
        // 1. Test corto circuito (non deve sollevare errore per variabile indefinita)
        let source_sc = "importa italiano da translate\ncrea x = falso && indefinita\ncrea y = vero || indefinita\n";
        engine.run(source_sc);
        assert!(engine.interpreter.exception.is_none());

        // 2. Test propagazione eccezione try-finally senza catch
        let source_tf = "importa italiano da translate\nprova {\nlancia \"errore_test\"\n} infine {\n}\n";
        engine.run(source_tf);
        assert!(engine.interpreter.exception.is_some());
        
        // 3. Test notazione con punto
        let source_dot = "importa italiano da translate\ncrea mappa = { \"nome\": \"mario\" }\ncrea nome_val = mappa.nome\n";
        engine.run(source_dot);
        assert!(engine.interpreter.exception.is_none());
        assert_eq!(engine.interpreter.get_var("nome_val"), crate::engine::value::Value::String("mario".to_string()));
    }
}