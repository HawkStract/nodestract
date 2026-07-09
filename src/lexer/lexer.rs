use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keyword generico contenente il nome inglese canonico (es. "let", "if", "sin")
    Keyword(String),

    // Identificatori e letterali
    Identifier(String), StringLiteral(String), Number(f64),

    // Delimitatori (caricati da delimiters.json)
    Delimiter(String),

    // Operatori (caricati da operators.json)
    Operator(String),

    EOF,
    Unknown(char),
}

impl Token {
    /// Restituisce la rappresentazione testuale del token.
    pub fn to_string_repr(&self) -> String {
        match self {
            Token::Keyword(s) => s.clone(),
            Token::Identifier(s) => s.clone(),
            Token::StringLiteral(s) => format!("\"{}\"", s),
            Token::Number(n) => n.to_string(),
            Token::Delimiter(s) => s.clone(),
            Token::Operator(s) => s.clone(),
            Token::EOF => "EOF".to_string(),
            Token::Unknown(c) => c.to_string(),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    // Operatori ordinati per lunghezza decrescente per matchare prima quelli più lunghi (es. "==" prima di "=")
    operators: Vec<(String, String)>,
    // Delimitatori ordinati per lunghezza decrescente
    delimiters: Vec<(String, String)>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        // Carica delimitatori e operatori dai JSON a tempo di compilazione
        let delimiters_json = include_str!("delimiters.json");
        let operators_json = include_str!("operators.json");

        let delimiters_map: HashMap<String, String> = serde_json::from_str(delimiters_json).unwrap_or_default();
        let operators_map: HashMap<String, String> = serde_json::from_str(operators_json).unwrap_or_default();

        // Ordina per lunghezza decrescente per garantire la corrispondenza più lunga (longest-match)
        let mut delimiters: Vec<(String, String)> = delimiters_map.into_iter().collect();
        delimiters.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        let mut operators: Vec<(String, String)> = operators_map.into_iter().collect();
        operators.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        Self {
            input: input.chars().collect(),
            position: 0,
            operators,
            delimiters,
        }
    }

    /// Converte la stringa sorgente in un vettore di Token.
    pub fn tokenize(&mut self, translation: &crate::engine::translate::TranslationEngine, filtered_engine: &crate::engine::filter::FilteredEngine) -> Vec<Token> {
        let mut tokens = Vec::new();
        while self.position < self.input.len() {
            let char = self.input[self.position];

            // 1. Salta gli spazi bianchi
            if char.is_whitespace() {
                self.position += 1;
                continue;
            }

            // 2. Salta i commenti
            if self.peek_str("//") {
                self.skip_comment();
                continue;
            }
            if self.peek_str("/*") {
                self.skip_multiline_comment();
                continue;
            }

            // 3. Cerca di fare match con gli operatori (dal più lungo)
            let mut matched_op = None;
            for (op_symbol, _) in &self.operators {
                if self.peek_str(op_symbol) {
                    matched_op = Some(op_symbol.clone());
                    break;
                }
            }
            if let Some(op) = matched_op {
                self.position += op.chars().count();
                tokens.push(Token::Operator(op));
                continue;
            }

            // 4. Cerca di fare match con i delimitatori (dal più lungo)
            let mut matched_delim = None;
            for (delim_symbol, _) in &self.delimiters {
                if self.peek_str(delim_symbol) {
                    matched_delim = Some(delim_symbol.clone());
                    break;
                }
            }
            if let Some(delim) = matched_delim {
                self.position += delim.chars().count();
                tokens.push(Token::Delimiter(delim));
                continue;
            }

            // 5. Riconosce le stringhe letterali
            if char == '"' {
                tokens.push(self.read_string());
                continue;
            }

            // 6. Riconosce i numeri
            if char.is_numeric() {
                tokens.push(self.read_number());
                continue;
            }

            // 7. Riconosce identificatori/keyword
            if char.is_alphabetic() || char == '_' {
                tokens.push(self.read_identifier(translation, filtered_engine));
                continue;
            }

            // 8. Carattere sconosciuto
            tokens.push(Token::Unknown(char));
            self.position += 1;
        }

        tokens.push(Token::EOF);
        tokens
    }

    fn peek_str(&self, prefix: &str) -> bool {
        let prefix_chars: Vec<char> = prefix.chars().collect();
        if self.position + prefix_chars.len() > self.input.len() {
            return false;
        }
        for (i, c) in prefix_chars.iter().enumerate() {
            if self.input[self.position + i] != *c {
                return false;
            }
        }
        true
    }

    fn read_identifier(&mut self, translation: &crate::engine::translate::TranslationEngine, filtered_engine: &crate::engine::filter::FilteredEngine) -> Token {
        let start = self.position;
        while self.position < self.input.len() && (self.input[self.position].is_alphanumeric() || self.input[self.position] == '_') {
            self.position += 1;
        }
        let text: String = self.input[start..self.position].iter().collect();

        // Traduce il keyword usando il motore filtrato
        if let Some(canonical) = filtered_engine.lookup(&text, translation) {
            Token::Keyword(canonical.to_string())
        } else {
            Token::Identifier(text)
        }
    }

    fn read_string(&mut self) -> Token {
        self.position += 1; 
        let mut text = String::new();
        while self.position < self.input.len() {
            let c = self.input[self.position];
            if c == '\\' {
                self.position += 1;
                if self.position < self.input.len() {
                    let escaped_char = self.input[self.position];
                    match escaped_char {
                        'n' => text.push('\n'), 't' => text.push('\t'), 'r' => text.push('\r'),
                        '"' => text.push('"'), '\\' => text.push('\\'),
                        _ => text.push(escaped_char),
                    }
                }
            } else if c == '"' { break; } 
            else { text.push(c); }
            self.position += 1;
        }
        self.position += 1;
        Token::StringLiteral(text)
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;
        let mut has_dot = false;
        while self.position < self.input.len() {
            let c = self.input[self.position];
            if c.is_numeric() { self.position += 1; } 
            else if c == '.' && !has_dot { has_dot = true; self.position += 1; } 
            else { break; }
        }
        let text: String = self.input[start..self.position].iter().collect();
        let value = text.parse::<f64>().unwrap_or(0.0);
        Token::Number(value)
    }

    fn skip_comment(&mut self) {
        while self.position < self.input.len() && self.input[self.position] != '\n' { self.position += 1; }
    }

    fn skip_multiline_comment(&mut self) {
        self.position += 2;
        while self.position < self.input.len() {
            if self.input[self.position] == '*' && self.peek_next() == '/' { self.position += 2; return; }
            self.position += 1;
        }
    }

    fn peek_next(&self) -> char {
        if self.position + 1 >= self.input.len() { return '\0'; }
        self.input[self.position + 1]
    }
}