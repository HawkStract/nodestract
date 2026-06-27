use std::collections::{HashMap, HashSet};
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
                entry.value = value;
                return;
            }
        }
        println!("Runtime Error: Variable '{}' not declared before assignment.", name);
    }

    pub fn define_var(&mut self, name: String, value: Value, is_mutable: bool) {
        self.current_scope().insert(name, VarEntry { value, is_mutable });
    }
}