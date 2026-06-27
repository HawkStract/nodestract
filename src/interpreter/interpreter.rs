use std::collections::{HashMap, HashSet};
use std::env;
use crate::engine::ast::{Program, Statement};
use crate::engine::value::Value;
use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use aes_gcm::aead::rand_core::RngCore;

pub mod expressions;
pub mod statements;
pub mod ops;
pub mod functions;
pub mod fs;
pub mod net;

#[derive(Clone, Debug)]
pub struct VarEntry {
    pub value: Value,
    pub is_mutable: bool,
    pub is_secure: bool,
}

pub struct Interpreter {
    pub scopes: Vec<HashMap<String, VarEntry>>,
    pub functions: HashMap<String, Statement>,
    pub loaded_files: HashSet<String>,
    pub last_return: Option<Value>,
    pub loop_break: bool,
    pub loop_continue: bool,
    pub exception: Option<Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            loaded_files: HashSet::new(),
            last_return: None,
            loop_break: false,
            loop_continue: false,
            exception: None,
        }
    }

    /// Load top-level definitions like functions
    pub fn load_program(&mut self, program: Program) {
        for stmt in &program.statements {
            match stmt {
                Statement::FunctionDecl { name, .. } => {
                    self.functions.insert(name.clone(), stmt.clone());
                }
                _ => {
                    self.execute_statement(stmt);
                }
            }
        }
    }

    /// Runs the program execution beginning from the main function
    pub fn run(&mut self, program: Program) {
        self.load_program(program);
        if let Some(func_stmt) = self.functions.get("main").cloned() {
            if let Statement::FunctionDecl { body, .. } = func_stmt {
                for s in body {
                    self.execute_statement(&s);
                    if self.exception.is_some() {
                        break;
                    }
                }
            }
        }
    }

    pub fn current_scope(&mut self) -> &mut HashMap<String, VarEntry> {
        self.scopes.last_mut().unwrap()
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn get_var(&self, name: &str) -> Value {
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return entry.value.clone();
            }
        }
        Value::Null
    }

    pub fn set_var(&mut self, name: String, value: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(entry) = scope.get_mut(&name) {
                if !entry.is_mutable {
                    println!("Runtime Error: Cannot assign to lock (constant) '{}'.", name);
                    return;
                }
                let final_val = if entry.is_secure {
                    let enc = Self::encrypt_vault(&value.to_string());
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

    pub fn define_var(&mut self, name: String, value: Value, is_mutable: bool, is_secure: bool) {
        let final_val = if is_secure {
            let s = if let Value::String(ref enc_str) = value {
                if enc_str.starts_with("ENC:") {
                    Self::decrypt_vault(enc_str)
                } else {
                    value.to_string()
                }
            } else {
                value.to_string()
            };
            Value::String(Self::encrypt_vault(&s))
        } else {
            if let Value::String(ref enc_str) = value {
                if enc_str.starts_with("ENC:") {
                    let decrypted = Self::decrypt_vault(enc_str);
                    if let Ok(i) = decrypted.parse::<i64>() {
                        Value::Integer(i)
                    } else if let Ok(f) = decrypted.parse::<f64>() {
                        Value::Float(f)
                    } else {
                        Value::String(decrypted)
                    }
                } else {
                    value
                }
            } else {
                value
            }
        };
        self.current_scope().insert(name, VarEntry { value: final_val, is_mutable, is_secure });
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

    pub fn encrypt_vault(val: &str) -> String {
        let key = Self::get_key();
        let cipher = Aes256Gcm::new(&key.into());
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        match cipher.encrypt(nonce, val.as_bytes()) {
            Ok(ciphertext) => format!("ENC:{}:{}", hex::encode(nonce_bytes), hex::encode(ciphertext)),
            Err(_) => "ERROR_ENCRYPT".to_string(),
        }
    }

    pub fn decrypt_vault(val: &str) -> String {
        let parts: Vec<&str> = val.split(':').collect();
        if parts.len() != 3 {
            return "ERROR_FORMAT".to_string();
        }
        let key = Self::get_key();
        let cipher = Aes256Gcm::new(&key.into());
        let nonce_vec = hex::decode(parts[1]).unwrap_or_default();
        let nonce = Nonce::from_slice(&nonce_vec);
        match cipher.decrypt(nonce, hex::decode(parts[2]).unwrap_or_default().as_ref()) {
            Ok(plaintext) => String::from_utf8(plaintext).unwrap_or_else(|_| "ERROR_UTF8".to_string()),
            Err(_) => "ERROR_DECRYPT".to_string(),
        }
    }
}