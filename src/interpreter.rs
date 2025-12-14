use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use crate::ast::{Program, Statement, Expression};
use crate::value::Value;

use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use aes_gcm::aead::rand_core::RngCore;

#[derive(Clone, Debug)]
struct VarEntry {
    value: Value,
    is_mutable: bool,
    is_secure: bool,
}

pub struct Interpreter {
    scopes: Vec<HashMap<String, VarEntry>>,
    capabilities: Vec<String>,
    functions: HashMap<String, Statement>,
    last_return: Option<Value>,
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

    fn current_scope(&mut self) -> &mut HashMap<String, VarEntry> {
        self.scopes.last_mut().unwrap()
    }

    fn get_var(&self, name: &str) -> Value {
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.get(name) {
                if let Value::String(s) = &entry.value {
                    if s.starts_with("ENC:") {
                        let decrypted = Self::decrypt_vault(s);
                        return Value::String(decrypted);
                    }
                }
                return entry.value.clone();
            }
        }
        Value::Null
    }

    fn set_var(&mut self, name: String, value: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(entry) = scope.get_mut(&name) {
                if !entry.is_mutable {
                    println!("Runtime Error: Cannot assign to lock (constant) '{}'.", name);
                    return; 
                }

                let final_val = if entry.is_secure {
                    let s = value.to_string(); 
                    let enc = Self::encrypt_vault(&s);
                    Value::String(enc)
                } else {
                    value
                };

                entry.value = final_val;
                return;
            }
        }
        println!("Runtime Error: Variable '{}' not declared before assignment.", name);
    }

    fn define_var(&mut self, name: String, value: Value, is_mutable: bool, is_secure: bool) {
        let final_val = if is_secure {
            let s = value.to_string();
            let enc = Self::encrypt_vault(&s);
            Value::String(enc)
        } else {
            value
        };
        
        let entry = VarEntry {
            value: final_val,
            is_mutable,
            is_secure,
        };
        
        self.current_scope().insert(name, entry);
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
            Statement::VarDecl { name, value, is_mutable, is_secure } => {
                let val = self.eval_expression(value);
                self.define_var(name.clone(), val, *is_mutable, *is_secure);
            }
            Statement::Assignment { name, value } => {
                let val = self.eval_expression(value);
                self.set_var(name.clone(), val);
            }
            Statement::IfStatement { condition, then_branch, else_branch } => {
                let cond_val = self.eval_expression(condition);
                if cond_val.is_truthy() {
                    for s in then_branch { self.execute_statement(s); }
                } else if let Some(else_stmts) = else_branch {
                    for s in else_stmts { self.execute_statement(s); }
                }
            }
            Statement::WhileStatement { condition, body } => {
                while self.eval_expression(condition).is_truthy() {
                    for s in body {
                        self.execute_statement(s);
                        if self.last_return.is_some() { return; }
                    }
                }
            }
            Statement::ForStatement { iterator, start, end, body } => {
                let start_val = self.eval_expression(start);
                let end_val = self.eval_expression(end);

                let start_int = match start_val {
                    Value::Integer(i) => i,
                    Value::Float(f) => f as i64,
                    _ => 0,
                };
                let end_int = match end_val {
                    Value::Integer(i) => i,
                    Value::Float(f) => f as i64,
                    _ => 0,
                };

                for i in start_int..end_int {
                    self.define_var(iterator.clone(), Value::Integer(i), false, false);
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

    fn eval_expression(&mut self, expr: &Expression) -> Value {
        match expr {
            Expression::LiteralStr(s) => Value::String(s.clone()),
            Expression::LiteralNum(n) => {
                if n.fract() == 0.0 {
                    Value::Integer(*n as i64)
                } else {
                    Value::Float(*n)
                }
            },
            Expression::Array(elements) => {
                let vals: Vec<Value> = elements.iter().map(|e| self.eval_expression(e)).collect();
                Value::Array(vals)
            },
            Expression::Map(pairs) => {
                let mut map = HashMap::new();
                for (k, v_expr) in pairs {
                    let val = self.eval_expression(v_expr);
                    map.insert(k.clone(), val);
                }
                Value::Map(map)
            },
            Expression::Variable(name) => self.get_var(name),
            Expression::Index { target, index } => {
                let target_val = self.eval_expression(target);
                let index_val = self.eval_expression(index);

                match target_val {
                    Value::Array(arr) => {
                        let idx = match index_val {
                            Value::Integer(i) => i as usize,
                            Value::Float(f) => f as usize,
                            _ => return Value::Null,
                        };
                        if idx < arr.len() { return arr[idx].clone(); }
                    },
                    Value::Map(map) => {
                        let key = index_val.to_string();
                        if let Some(val) = map.get(&key) {
                            return val.clone();
                        }
                    },
                    _ => {}
                }
                Value::Null
            },
            Expression::BinaryOp { left, operator, right } => {
                let l = self.eval_expression(left);
                let r = self.eval_expression(right);
                self.eval_binary_op(l, operator, r)
            },
            Expression::FunctionCall { target, args } => {
                self.handle_function_call(target, args)
            }
        }
    }

    fn eval_binary_op(&self, left: Value, operator: &str, right: Value) -> Value {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => match operator {
                "+" => Value::Integer(a + b),
                "-" => Value::Integer(a - b),
                "*" => Value::Integer(a * b),
                "/" => Value::Integer(a / b),
                ">" => Value::Boolean(a > b),
                "<" => Value::Boolean(a < b),
                "==" => Value::Boolean(a == b),
                _ => Value::Null,
            },
            (Value::Float(a), Value::Float(b)) => match operator {
                "+" => Value::Float(a + b),
                "-" => Value::Float(a - b),
                "*" => Value::Float(a * b),
                "/" => Value::Float(a / b),
                ">" => Value::Boolean(a > b),
                "<" => Value::Boolean(a < b),
                "==" => Value::Boolean(a == b),
                _ => Value::Null,
            },
            (Value::Integer(a), Value::Float(b)) => self.eval_binary_op(Value::Float(a as f64), operator, Value::Float(b)),
            (Value::Float(a), Value::Integer(b)) => self.eval_binary_op(Value::Float(a), operator, Value::Float(b as f64)),

            (Value::String(a), Value::String(b)) => match operator {
                "+" => Value::String(a + &b),
                "==" => Value::Boolean(a == b),
                _ => Value::Null,
            },
            (Value::String(a), b) => match operator {
                "+" => Value::String(format!("{}{}", a, b)),
                _ => Value::Null,
            },
            (a, Value::String(b)) => match operator {
                "+" => Value::String(format!("{}{}", a, b)),
                _ => Value::Null,
            },

            _ => Value::Null,
        }
    }

    fn handle_function_call(&mut self, target: &str, args: &Vec<Expression>) -> Value {
        if target.contains(".") {
            let service = target.split('.').next().unwrap_or("");
            
            if !self.capabilities.contains(&service.to_string()) && service != "Sys" && service != "Array" {
                println!("SECURITY ALERT: Capability '{}' blocked for '{}'. Execution Halted.", service, target);
                std::process::exit(1);
            }

            match target {
                "IO.print" => {
                    let output: Vec<String> = args.iter()
                        .map(|a| self.eval_expression(a).to_string())
                        .collect();
                    println!("{}", output.join(" "));
                    return Value::Null;
                },
                "IO.input" => {
                    if let Some(prompt_expr) = args.get(0) {
                        print!("{}", self.eval_expression(prompt_expr));
                        io::stdout().flush().unwrap();
                    }
                    let mut buffer = String::new();
                    io::stdin().read_line(&mut buffer).unwrap();
                    return Value::String(buffer.trim().to_string());
                },
                "Array.len" => {
                    if let Some(arg) = args.get(0) {
                        if let Value::Array(arr) = self.eval_expression(arg) {
                            return Value::Integer(arr.len() as i64);
                        }
                    }
                    return Value::Integer(0);
                },
                "Array.push" => {
                    if args.len() >= 2 {
                        let mut arr_val = self.eval_expression(&args[0]);
                        let val_to_push = self.eval_expression(&args[1]);
                        
                        if let Value::Array(ref mut arr) = arr_val {
                            arr.push(val_to_push);
                            return Value::Array(arr.clone());
                        }
                    }
                    return Value::Null;
                },
                "Sys.memory_dump" => {
                    if let Some(Expression::Variable(var_name)) = args.get(0) {
                        let val = self.get_var(var_name);
                        println!("[RAM DUMP] Variable '{}' -> {:?}", var_name, val);
                    }
                    return Value::Null;
                }
                _ => {}
            }
        }

        if let Some(func_stmt) = self.functions.get(target).cloned() {
            if let Statement::FunctionDecl { params, body, .. } = func_stmt {
                let mut new_scope = HashMap::new();
                for (i, param_name) in params.iter().enumerate() {
                    let arg_val = if i < args.len() {
                        self.eval_expression(&args[i])
                    } else {
                        Value::Null
                    };
                    
                    let entry = VarEntry {
                        value: arg_val,
                        is_mutable: true,
                        is_secure: false,
                    };
                    new_scope.insert(param_name.clone(), entry);
                }

                self.scopes.push(new_scope);
                for s in body {
                    self.execute_statement(&s);
                    if self.last_return.is_some() { break; }
                }
                self.scopes.pop();
                
                let result = self.last_return.clone().unwrap_or(Value::Null);
                self.last_return = None;
                return result;
            }
        }
        
        Value::Null
    }
}