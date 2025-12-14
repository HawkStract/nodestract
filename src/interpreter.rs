use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use crate::ast::{Program, Statement, Expression};
use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use aes_gcm::aead::rand_core::RngCore;

pub struct Interpreter {
    scopes: Vec<HashMap<String, String>>,
    capabilities: Vec<String>,
    functions: HashMap<String, Statement>,
    last_return: Option<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        let global_scope = HashMap::new();
        Self {
            scopes: vec![global_scope],
            capabilities: Vec::new(),
            functions: HashMap::new(),
            last_return: None,
        }
    }

    fn current_scope(&mut self) -> &mut HashMap<String, String> {
        self.scopes.last_mut().unwrap()
    }

    fn get_var(&self, name: &str) -> String {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                if val.starts_with("ENC:") {
                    return Self::decrypt_vault(val);
                }
                return val.clone();
            }
        }
        "undefined".to_string()
    }

    fn get_raw_memory(&self, name: &str) -> String {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return val.clone();
            }
        }
        "undefined".to_string()
    }

    fn set_var(&mut self, name: String, value: String) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name) {
                let is_encrypted = scope.get(&name).map(|v| v.starts_with("ENC:")).unwrap_or(false);
                let final_val = if is_encrypted { Self::encrypt_vault(&value) } else { value };
                scope.insert(name, final_val);
                return;
            }
        }
        println!("Error: Variable '{}' not declared before assignment.", name);
    }

    fn define_var(&mut self, name: String, value: String, is_secure: bool) {
        let final_val = if is_secure { Self::encrypt_vault(&value) } else { value };
        self.current_scope().insert(name, final_val);
    }

    fn get_key() -> [u8; 32] {
        let key_str = env::var("NSC_VAULT_KEY").unwrap_or_else(|_| "HAWK_MASTER_KEY_2025_SECURE_AES_".to_string());
        let mut key_bytes = [0u8; 32];
        let bytes = key_str.as_bytes();
        for (i, b) in bytes.iter().enumerate().take(32) {
            key_bytes[i] = *b;
        }
        key_bytes
    }

    fn encrypt_vault(val: &str) -> String {
        let key = Self::get_key();
        let cipher = Aes256Gcm::new(&key.into());
        
        let mut nonce_bytes = [0u8; 12];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        match cipher.encrypt(nonce, val.as_bytes()) {
            Ok(ciphertext) => {
                let nonce_hex = hex::encode(nonce_bytes);
                let cipher_hex = hex::encode(ciphertext);
                format!("ENC:{}:{}", nonce_hex, cipher_hex)
            },
            Err(_) => "ERROR_ENCRYPT".to_string()
        }
    }

    fn decrypt_vault(val: &str) -> String {
        let parts: Vec<&str> = val.split(':').collect();
        if parts.len() != 3 { return "ERROR_FORMAT".to_string(); }

        let nonce_hex = parts[1];
        let cipher_hex = parts[2];

        let key = Self::get_key();
        let cipher = Aes256Gcm::new(&key.into());

        let nonce_bytes = hex::decode(nonce_hex).unwrap_or_default();
        let ciphertext = hex::decode(cipher_hex).unwrap_or_default();

        if nonce_bytes.len() != 12 { return "ERROR_NONCE".to_string(); }
        
        let nonce = Nonce::from_slice(&nonce_bytes);

        match cipher.decrypt(nonce, ciphertext.as_ref()) {
            Ok(plaintext) => String::from_utf8(plaintext).unwrap_or_else(|_| "ERROR_UTF8".to_string()),
            Err(_) => "ERROR_DECRYPT".to_string()
        }
    }

    // --- ARRAY HELPERS ---
    // Poiché per ora lavoriamo con Stringhe in memoria, serializziamo le liste come "[1, 2, 3]"
    fn parse_array_str(val: &str) -> Vec<String> {
        let trimmed = val.trim();
        if !trimmed.starts_with('[') || !trimmed.ends_with(']') { return vec![]; }
        let content = &trimmed[1..trimmed.len()-1];
        if content.is_empty() { return vec![]; }
        content.split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }

    fn build_array_str(elems: Vec<String>) -> String {
        format!("[{}]", elems.join(", "))
    }

    pub fn run(&mut self, program: Program) {
        for stmt in &program.statements {
            match stmt {
                Statement::CapabilityUse { service, .. } => {
                    self.capabilities.push(service.clone());
                },
                Statement::FunctionDecl { name, .. } => {
                    self.functions.insert(name.clone(), stmt.clone());
                },
                Statement::VarDecl { .. } => {
                    self.execute_statement(stmt);
                },
                _ => {}
            }
        }

        if let Some(func_stmt) = self.functions.get("main").cloned() {
             if let Statement::FunctionDecl { body, .. } = func_stmt {
                 for s in body {
                     self.execute_statement(&s);
                 }
             }
        } else {
            println!("Runtime Error: No 'main' function found.");
        }
    }

    fn execute_statement(&mut self, stmt: &Statement) {
        if self.last_return.is_some() { return; }

        match stmt {
            Statement::VarDecl { name, value, is_secure, .. } => {
                let val = self.eval_expression(value);
                self.define_var(name.clone(), val, *is_secure);
            }
            Statement::Assignment { name, value } => {
                let val = self.eval_expression(value);
                self.set_var(name.clone(), val);
            }
            Statement::IfStatement { condition, then_branch, else_branch } => {
                let cond_val = self.eval_expression(condition);
                if cond_val == "true" {
                    for s in then_branch { self.execute_statement(s); }
                } else if let Some(else_stmts) = else_branch {
                    for s in else_stmts { self.execute_statement(s); }
                }
            }
            Statement::WhileStatement { condition, body } => {
                while self.eval_expression(condition) == "true" {
                    for s in body {
                        self.execute_statement(s);
                        if self.last_return.is_some() { return; }
                    }
                }
            }
            Statement::ForStatement { iterator, start, end, body } => {
                let start_val = self.eval_expression(start).parse::<i64>().unwrap_or(0);
                let end_val = self.eval_expression(end).parse::<i64>().unwrap_or(0);

                for i in start_val..end_val {
                    self.define_var(iterator.clone(), i.to_string(), false);
                    for s in body {
                        self.execute_statement(s);
                        if self.last_return.is_some() { return; }
                    }
                }
            }
            Statement::ReturnStatement { value } => {
                self.last_return = Some(self.eval_expression(value));
            }
            Statement::Expr(expr) => {
                self.eval_expression(expr);
            }
            _ => {}
        }
    }

    fn eval_expression(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::LiteralStr(s) => s.clone(),
            Expression::LiteralNum(n) => n.to_string(),
            // NUOVO: Creazione Array
            Expression::Array(elements) => {
                let vals: Vec<String> = elements.iter().map(|e| self.eval_expression(e)).collect();
                Self::build_array_str(vals)
            }
            Expression::Variable(name) => self.get_var(name),
            // NUOVO: Accesso Indice (list[0])
            Expression::Index { target, index } => {
                let arr_str = self.eval_expression(target);
                let idx_val = self.eval_expression(index).parse::<usize>().unwrap_or(0);
                let elems = Self::parse_array_str(&arr_str);
                
                if idx_val < elems.len() {
                    elems[idx_val].clone()
                } else {
                    "undefined".to_string()
                }
            },
            Expression::BinaryOp { left, operator, right } => {
                let l_val = self.eval_expression(left);
                let r_val = self.eval_expression(right);
                
                let l_num = l_val.parse::<f64>();
                let r_num = r_val.parse::<f64>();

                if let (Ok(l), Ok(r)) = (l_num, r_num) {
                    match operator.as_str() {
                        "+" => (l + r).to_string(),
                        "-" => (l - r).to_string(),
                        "*" => (l * r).to_string(),
                        "/" => (l / r).to_string(),
                        ">" => (l > r).to_string(),
                        "<" => (l < r).to_string(),
                        "==" => (l == r).to_string(),
                        _ => "0".to_string(),
                    }
                } else {
                    if operator == "+" {
                        format!("{}{}", l_val, r_val)
                    } else if operator == "==" {
                         (l_val == r_val).to_string()
                    } else {
                        "NaN".to_string()
                    }
                }
            }
            Expression::FunctionCall { target, args } => {
                if target.contains(".") {
                     let service = target.split('.').next().unwrap_or("");
                     
                     if !self.capabilities.contains(&service.to_string()) && service != "Sys" && service != "Array" {
                        println!("SECURITY ALERT: Capability '{}' blocked for '{}'. Execution Halted.", service, target);
                        std::process::exit(1);
                    }

                    if target == "IO.print" {
                        let output: Vec<String> = args.iter().map(|a| self.eval_expression(a)).collect();
                        println!("{}", output.join(" "));
                        return String::new();
                    }
                    
                    // NUOVO: Input Utente
                    if target == "IO.input" {
                        if let Some(prompt_expr) = args.get(0) {
                            print!("{}", self.eval_expression(prompt_expr));
                            io::stdout().flush().unwrap();
                        }
                        let mut buffer = String::new();
                        io::stdin().read_line(&mut buffer).unwrap();
                        return buffer.trim().to_string();
                    }

                    // NUOVO: Array Utils
                    if target == "Array.len" {
                        let arr = self.eval_expression(&args[0]);
                        return Self::parse_array_str(&arr).len().to_string();
                    }
                    if target == "Array.push" {
                        let arr = self.eval_expression(&args[0]);
                        let val = self.eval_expression(&args[1]);
                        let mut elems = Self::parse_array_str(&arr);
                        elems.push(val);
                        return Self::build_array_str(elems);
                    }
                    
                    if target == "Sys.memory_dump" {
                        if let Some(Expression::Variable(var_name)) = args.get(0) {
                            let raw_val = self.get_raw_memory(var_name);
                            println!("[RAM DUMP] Variable '{}' -> '{}'", var_name, raw_val);
                        }
                        return String::new();
                    }
                }

                if let Some(func_stmt) = self.functions.get(target).cloned() {
                    if let Statement::FunctionDecl { params, body, .. } = func_stmt {
                        let mut new_scope = HashMap::new();
                        for (i, param_name) in params.iter().enumerate() {
                            let arg_val = if i < args.len() {
                                self.eval_expression(&args[i])
                            } else {
                                "undefined".to_string()
                            };
                            new_scope.insert(param_name.clone(), arg_val);
                        }

                        self.scopes.push(new_scope);
                        for s in body {
                            self.execute_statement(&s);
                            if self.last_return.is_some() { break; }
                        }
                        self.scopes.pop();
                        
                        let result = self.last_return.clone().unwrap_or_else(|| "undefined".to_string());
                        self.last_return = None;
                        return result;
                    }
                }
                
                "undefined".to_string()
            }
        }
    }
}