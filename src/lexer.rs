#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Lock,
    Stract,
    Vault,
    Safe,
    Capability,
    Use,
    Module,
    Func,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Identifier(String),
    StringLiteral(String),
    Number(f64),
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Equal,
    EqualEqual,
    Greater,
    Less,
    Plus,
    Minus,
    Star,
    Slash,
    Dot,
    Range,
    Comma,
    Colon,
    EOF,
    Unknown(char),
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            let char = self.input[self.position];

            match char {
                ' ' | '\t' | '\n' | '\r' => {
                    self.position += 1;
                }
                '{' => { tokens.push(Token::LeftBrace); self.position += 1; }
                '}' => { tokens.push(Token::RightBrace); self.position += 1; }
                '(' => { tokens.push(Token::LeftParen); self.position += 1; }
                ')' => { tokens.push(Token::RightParen); self.position += 1; }
                '[' => { tokens.push(Token::LeftBracket); self.position += 1; }
                ']' => { tokens.push(Token::RightBracket); self.position += 1; }
                '.' => { 
                    if self.peek_next() == '.' {
                        self.position += 2;
                        tokens.push(Token::Range);
                    } else {
                        tokens.push(Token::Dot); 
                        self.position += 1; 
                    }
                }
                ',' => { tokens.push(Token::Comma); self.position += 1; }
                ':' => { tokens.push(Token::Colon); self.position += 1; }
                '+' => { tokens.push(Token::Plus); self.position += 1; }
                '-' => { tokens.push(Token::Minus); self.position += 1; }
                '*' => { tokens.push(Token::Star); self.position += 1; }
                '/' => {
                    if self.peek_next() == '*' {
                        self.skip_multiline_comment();
                    } else if self.peek_next() == '/' {
                        self.skip_comment();
                    } else {
                        tokens.push(Token::Slash);
                        self.position += 1;
                    }
                }
                '=' => {
                    if self.peek_next() == '=' {
                        self.position += 2;
                        tokens.push(Token::EqualEqual);
                    } else {
                        tokens.push(Token::Equal);
                        self.position += 1;
                    }
                }
                '>' => { tokens.push(Token::Greater); self.position += 1; }
                '<' => { tokens.push(Token::Less); self.position += 1; }
                '"' => {
                    tokens.push(self.read_string());
                }
                _ if char.is_alphabetic() => {
                    tokens.push(self.read_identifier());
                }
                _ if char.is_numeric() => {
                    tokens.push(self.read_number());
                }
                _ => {
                    tokens.push(Token::Unknown(char));
                    self.position += 1;
                }
            }
        }
        tokens.push(Token::EOF);
        tokens
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;
        while self.position < self.input.len() && (self.input[self.position].is_alphanumeric() || self.input[self.position] == '_') {
            self.position += 1;
        }
        
        let text: String = self.input[start..self.position].iter().collect();
        match text.as_str() {
            "lock" => Token::Lock,
            "stract" => Token::Stract,
            "vault" => Token::Vault,
            "safe" => Token::Safe,
            "capability" => Token::Capability,
            "use" => Token::Use,
            "module" => Token::Module,
            "func" => Token::Func,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "in" => Token::In,
            "return" => Token::Return,
            _ => Token::Identifier(text),
        }
    }

    fn read_string(&mut self) -> Token {
        self.position += 1;
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position] != '"' {
            self.position += 1;
        }
        let text: String = self.input[start..self.position].iter().collect();
        self.position += 1;
        Token::StringLiteral(text)
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;
        let mut has_dot = false;

        while self.position < self.input.len() {
            let c = self.input[self.position];
            if c.is_numeric() {
                self.position += 1;
            } else if c == '.' && !has_dot {
                has_dot = true;
                self.position += 1;
            } else {
                break;
            }
        }

        let text: String = self.input[start..self.position].iter().collect();
        let value = text.parse::<f64>().unwrap_or(0.0);
        Token::Number(value)
    }

    fn skip_comment(&mut self) {
        while self.position < self.input.len() && self.input[self.position] != '\n' {
            self.position += 1;
        }
    }

    fn skip_multiline_comment(&mut self) {
        self.position += 2;
        while self.position < self.input.len() {
            if self.input[self.position] == '*' && self.peek_next() == '/' {
                self.position += 2;
                return;
            }
            self.position += 1;
        }
    }

    fn peek_next(&self) -> char {
        if self.position + 1 >= self.input.len() {
            return '\0';
        }
        self.input[self.position + 1]
    }
}