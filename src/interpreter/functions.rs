use crate::engine::ast::{Statement, Expression};
use crate::engine::value::Value;
use super::{Interpreter, VarEntry};
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use super::fs;
use super::net;

impl Interpreter {
    pub fn handle_function_call(&mut self, target: &str, args: &Vec<Expression>) -> Value {
        if !self.is_function_defined(target) {
            let err_msg = format!("Runtime Error: Function '{}' is not defined", target);
            self.exception = Some(Value::String(err_msg));
            return Value::Null;
        }
        if !self.is_function_arity_valid(target, args.len()) {
            let err_msg = format!(
                "Arity Mismatch: Function '{}' expects different number of arguments (provided {})",
                target, args.len()
            );
            self.exception = Some(Value::String(err_msg));
            return Value::Null;
        }

        match target {
            // Operazioni di I/O
            "print" => {
                let mut output = String::new();
                for a in args {
                    let val = self.eval_expression(a);
                    output.push_str(&val.to_string());
                }
                println!("{}", output);
                let _ = io::stdout().flush();
                Value::Null
            }
            "input" => {
                if let Some(prompt_expr) = args.get(0) {
                    let raw_prompt = self.eval_expression(prompt_expr);
                    print!("{}", raw_prompt);
                    let _ = io::stdout().flush();
                }
                let mut buffer = String::new();
                if io::stdin().read_line(&mut buffer).is_ok() {
                    Value::String(buffer.trim_end().to_string())
                } else {
                    Value::Null
                }
            }

            // Operazioni su File System
            "read" => {
                if let Some(path_expr) = args.get(0) {
                    let path_val = self.eval_expression(path_expr);
                    match fs::read_file(&path_val.to_string()) {
                        Ok(val) => val,
                        Err(e) => {
                            self.exception = Some(Value::String(e));
                            Value::Null
                        }
                    }
                } else {
                    Value::Null
                }
            }
            "write" => {
                if args.len() >= 2 {
                    let path_val = self.eval_expression(&args[0]);
                    let content_val = self.eval_expression(&args[1]);
                    match fs::write_file(&path_val.to_string(), &content_val) {
                        Ok(val) => val,
                        Err(e) => {
                            self.exception = Some(Value::String(e));
                            Value::Boolean(false)
                        }
                    }
                } else {
                    Value::Boolean(false)
                }
            }
            "delete" => {
                if let Some(path_expr) = args.get(0) {
                    let path_val = self.eval_expression(path_expr);
                    match fs::delete_file(&path_val.to_string()) {
                        Ok(val) => val,
                        Err(e) => {
                            self.exception = Some(Value::String(e));
                            Value::Boolean(false)
                        }
                    }
                } else {
                    Value::Boolean(false)
                }
            }

            // Operazioni matematiche
            "sin" => {
                if let Some(arg) = args.get(0) {
                    let val = self.eval_expression(arg);
                    let num = match val {
                        Value::Integer(i) => i as f64,
                        Value::Float(f) => f,
                        _ => 0.0,
                    };
                    Value::Float(num.sin())
                } else {
                    Value::Null
                }
            }
            "cos" => {
                if let Some(arg) = args.get(0) {
                    let val = self.eval_expression(arg);
                    let num = match val {
                        Value::Integer(i) => i as f64,
                        Value::Float(f) => f,
                        _ => 0.0,
                    };
                    Value::Float(num.cos())
                } else {
                    Value::Null
                }
            }
            "sqrt" => {
                if let Some(arg) = args.get(0) {
                    let val = self.eval_expression(arg);
                    let num = match val {
                        Value::Integer(i) => i as f64,
                        Value::Float(f) => f,
                        _ => 0.0,
                    };
                    Value::Float(num.sqrt())
                } else {
                    Value::Null
                }
            }
            "random" => {
                Value::Float(fastrand::f64())
            }
            "round" => {
                if let Some(arg) = args.get(0) {
                    let val = self.eval_expression(arg);
                    let num = match val {
                        Value::Integer(i) => return Value::Integer(i),
                        Value::Float(f) => f.round() as i64,
                        _ => 0,
                    };
                    Value::Integer(num)
                } else {
                    Value::Null
                }
            }
            "min" => {
                if args.len() >= 2 {
                    let l = self.eval_expression(&args[0]);
                    let r = self.eval_expression(&args[1]);
                    match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.min(b)),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a.min(b)),
                        (Value::Integer(a), Value::Float(b)) => Value::Float((a as f64).min(b)),
                        (Value::Float(a), Value::Integer(b)) => Value::Float(a.min(b as f64)),
                        _ => Value::Null,
                    }
                } else {
                    Value::Null
                }
            }
            "max" => {
                if args.len() >= 2 {
                    let l = self.eval_expression(&args[0]);
                    let r = self.eval_expression(&args[1]);
                    match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a.max(b)),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a.max(b)),
                        (Value::Integer(a), Value::Float(b)) => Value::Float((a as f64).max(b)),
                        (Value::Float(a), Value::Integer(b)) => Value::Float(a.max(b as f64)),
                        _ => Value::Null,
                    }
                } else {
                    Value::Null
                }
            }
            "abs" => {
                if let Some(arg) = args.get(0) {
                    let val = self.eval_expression(arg);
                    match val {
                        Value::Integer(i) => Value::Integer(i.abs()),
                        Value::Float(f) => Value::Float(f.abs()),
                        _ => Value::Null,
                    }
                } else {
                    Value::Null
                }
            }
            "log" => {
                if let Some(arg) = args.get(0) {
                    let val = self.eval_expression(arg);
                    let num = match val {
                        Value::Integer(i) => i as f64,
                        Value::Float(f) => f,
                        _ => 0.0,
                    };
                    Value::Float(num.ln())
                } else {
                    Value::Null
                }
            }
            "pow" => {
                if args.len() >= 2 {
                    let base_val = self.eval_expression(&args[0]);
                    let exponent_val = self.eval_expression(&args[1]);
                    let base = match base_val {
                        Value::Integer(i) => i as f64,
                        Value::Float(f) => f,
                        _ => 0.0,
                    };
                    let exponent = match exponent_val {
                        Value::Integer(i) => i as f64,
                        Value::Float(f) => f,
                        _ => 0.0,
                    };
                    Value::Float(base.powf(exponent))
                } else {
                    Value::Null
                }
            }

            // Utility generali
            "len" => {
                if let Some(arg) = args.get(0) {
                    let val = self.eval_expression(arg);
                    match val {
                        Value::Array(arr) => Value::Integer(arr.len() as i64),
                        Value::String(s) => Value::Integer(s.len() as i64),
                        Value::Map(m) => Value::Integer(m.len() as i64),
                        _ => Value::Integer(0),
                    }
                } else {
                    Value::Integer(0)
                }
            }
            "sleep" => {
                if let Some(arg) = args.get(0) {
                    let val = self.eval_expression(arg);
                    let secs = match val {
                        Value::Integer(i) => i as u64,
                        Value::Float(f) => f as u64,
                        _ => 0,
                    };
                    thread::sleep(Duration::from_secs(secs));
                }
                Value::Null
            }
            "exit" => {
                if let Some(arg) = args.get(0) {
                    let val = self.eval_expression(arg);
                    let code = match val {
                        Value::Integer(i) => i as i32,
                        Value::Float(f) => f as i32,
                        _ => 0,
                    };
                    std::process::exit(code);
                }
                std::process::exit(0);
            }

            // Operazioni di rete
            "fetch" => {
                if let Some(url_expr) = args.get(0) {
                    let url_val = self.eval_expression(url_expr);
                    match net::get(&url_val.to_string()) {
                        Ok(val) => val,
                        Err(e) => {
                            self.exception = Some(Value::String(e));
                            Value::Null
                        }
                    }
                } else {
                    Value::Null
                }
            }
            "send" => {
                if args.len() >= 2 {
                    let url_val = self.eval_expression(&args[0]);
                    let body_val = self.eval_expression(&args[1]);
                    match net::post(&url_val.to_string(), &body_val.to_string()) {
                        Ok(val) => val,
                        Err(e) => {
                            self.exception = Some(Value::String(e));
                            Value::Null
                        }
                    }
                } else {
                    Value::Null
                }
            }

            _ => {
                if let Some(func_stmt) = self.functions.get(target).cloned() {
                    if let Statement::FunctionDecl { params, body, .. } = func_stmt {
                        let mut new_scope = HashMap::new();
                        for (i, param_name) in params.iter().enumerate() {
                            let arg_val = self.eval_expression(&args[i]);
                            let entry = VarEntry { value: arg_val, is_mutable: true };
                            new_scope.insert(param_name.clone(), entry);
                        }

                        let scope_idx = self.scopes.len();
                        self.scopes.push(new_scope);
                        self.fn_scope_starts.push(scope_idx);

                        for s in body {
                            self.execute_statement(&s);
                            if self.last_return.is_some() || self.exception.is_some() {
                                break;
                            }
                        }

                        self.fn_scope_starts.pop();
                        self.scopes.truncate(scope_idx);

                        let result = self.last_return.clone().unwrap_or(Value::Null);
                        self.last_return = None;
                        return result;
                    }
                }
                Value::Null
            }
        }
    }
}