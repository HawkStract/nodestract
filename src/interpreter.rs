use std::collections::HashMap;
use crate::ast::{Program, Statement, Expression};

pub struct Interpreter {
    variables: HashMap<String, String>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn run(&mut self, program: Program) {
        for stmt in &program.statements {
            if let Statement::VarDecl { name, value, .. } = stmt {
                let val = self.eval_expression(value);
                self.variables.insert(name.clone(), val);
            }
        }

        if let Some(Statement::FunctionDecl { body, .. }) = program.statements.iter().find(|s| is_main_function(s)) {
            for stmt in body {
                self.execute_statement(stmt);
            }
        } else {
            println!("Runtime Error: No 'main' function found.");
        }
    }

    fn execute_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VarDecl { name, value, .. } => {
                let val = self.eval_expression(value);
                self.variables.insert(name.clone(), val);
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
            Expression::Variable(name) => {
                self.variables.get(name).cloned().unwrap_or_else(|| "undefined".to_string())
            }
            Expression::BinaryOp { left, operator, right } => {
                let left_val = self.eval_expression(left);
                let right_val = self.eval_expression(right);
                if operator == "+" {
                    format!("{}{}", left_val, right_val)
                } else {
                    String::new()
                }
            }
            Expression::FunctionCall { target, args } => {
                if target == "IO.print" {
                    let output: Vec<String> = args.iter().map(|a| self.eval_expression(a)).collect();
                    println!("{}", output.join(" "));
                }
                String::new()
            }
            _ => String::new(),
        }
    }
}

fn is_main_function(stmt: &Statement) -> bool {
    match stmt {
        Statement::FunctionDecl { name, .. } => name == "main",
        _ => false,
    }
}