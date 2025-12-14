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
            if let Statement::VarDecl { .. } = stmt {
                self.execute_statement(stmt);
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
            Statement::Assignment { name, value } => {
                let val = self.eval_expression(value);
                // Aggiorna solo se esiste
                if self.variables.contains_key(name) {
                    self.variables.insert(name.clone(), val);
                } else {
                    println!("Error: Variable '{}' not declared before assignment.", name);
                }
            }
            Statement::IfStatement { condition, then_branch, else_branch } => {
                let cond_val = self.eval_expression(condition);
                if cond_val == "true" {
                    for s in then_branch { self.execute_statement(s); }
                } else if let Some(else_stmts) = else_branch {
                    for s in else_stmts { self.execute_statement(s); }
                }
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
            Expression::Variable(name) => {
                self.variables.get(name).cloned().unwrap_or_else(|| "undefined".to_string())
            }
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
                if target == "IO.print" {
                    let output: Vec<String> = args.iter().map(|a| self.eval_expression(a)).collect();
                    println!("{}", output.join(" "));
                }
                String::new()
            }
        }
    }
}

fn is_main_function(stmt: &Statement) -> bool {
    match stmt {
        Statement::FunctionDecl { name, .. } => name == "main",
        _ => false,
    }
}