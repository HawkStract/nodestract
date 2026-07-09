use std::collections::HashMap;
use crate::engine::ast::{Program, Statement};
use crate::engine::value::Value;

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
}

pub struct Interpreter {
    pub scopes: Vec<HashMap<String, VarEntry>>,
    pub fn_scope_starts: Vec<usize>,
    pub functions: HashMap<String, Statement>,
    pub last_return: Option<Value>,
    pub loop_break: bool,
    pub loop_continue: bool,
    pub exception: Option<Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            fn_scope_starts: Vec::new(),
            functions: HashMap::new(),
            last_return: None,
            loop_break: false,
            loop_continue: false,
            exception: None,
        }
    }

    /// Carica le definizioni globali come le funzioni
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

    /// Esegue il programma partendo dalla funzione "main" se presente, altrimenti esegue le istruzioni globali.
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
        let start_idx = self.fn_scope_starts.last().cloned().unwrap_or(0);
        for idx in (start_idx..self.scopes.len()).rev() {
            if let Some(entry) = self.scopes[idx].get(name) {
                return entry.value.clone();
            }
        }
        if start_idx > 0 {
            if let Some(entry) = self.scopes[0].get(name) {
                return entry.value.clone();
            }
        }
        Value::Null
    }



    pub fn define_var(&mut self, name: String, value: Value, is_mutable: bool) {
        self.current_scope().insert(name, VarEntry { value, is_mutable });
    }

    pub fn get_var_mut(&mut self, name: &str) -> Option<&mut VarEntry> {
        let start_idx = self.fn_scope_starts.last().cloned().unwrap_or(0);
        let mut target_idx = None;
        for idx in (start_idx..self.scopes.len()).rev() {
            if self.scopes[idx].contains_key(name) {
                target_idx = Some(idx);
                break;
            }
        }
        if let Some(idx) = target_idx {
            return self.scopes[idx].get_mut(name);
        }
        if start_idx > 0 {
            if self.scopes[0].contains_key(name) {
                return self.scopes[0].get_mut(name);
            }
        }
        None
    }

    pub fn is_function_arity_valid(&self, func_name: &str, args_count: usize) -> bool {
        match func_name {
            "print" => true,
            "input" => args_count <= 1,
            "read" | "delete" | "sin" | "cos" | "sqrt" | "round" | "abs" | "log" | "sleep" | "fetch" | "len" => args_count == 1,
            "write" | "min" | "max" | "pow" | "send" => args_count == 2,
            "random" => args_count == 0,
            "exit" => args_count <= 1,
            _ => {
                if let Some(Statement::FunctionDecl { params, .. }) = self.functions.get(func_name) {
                    params.len() == args_count
                } else {
                    false
                }
            }
        }
    }

    pub fn mutate_value_at_path(val: &mut Value, path: &[Value], new_val: Value) -> Result<(), String> {
        if path.is_empty() {
            *val = new_val;
            return Ok(());
        }
        match val {
            Value::Array(arr) => {
                let idx = match &path[0] {
                    Value::Integer(i) => *i as usize,
                    Value::Float(f) => *f as usize,
                    _ => return Err("L'indice dell'array deve essere un numero intero".to_string()),
                };
                if idx >= arr.len() {
                    return Err("Indice dell'array fuori dai limiti".to_string());
                }
                Self::mutate_value_at_path(&mut arr[idx], &path[1..], new_val)
            }
            Value::Map(map) => {
                let key = path[0].to_string();
                let entry = map.entry(key).or_insert(Value::Null);
                Self::mutate_value_at_path(entry, &path[1..], new_val)
            }
            _ => Err("Impossibile indicizzare questo tipo di dato".to_string()),
        }
    }
}