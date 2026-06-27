use crate::engine::ast::{Statement, Expression};
use crate::engine::value::Value;
use super::{Interpreter, VarEntry};
use std::io::{self, Write};
use super::fs;
use super::net; // <--- Import Net

impl Interpreter {
    pub fn handle_function_call(&mut self, target: &str, args: &Vec<Expression>) -> Value {
        if target.contains(".") {
            let service = target.split('.').next().unwrap_or("");
            
            // Check Capability
            if !self.capabilities.contains(&service.to_string()) && service != "Sys" && service != "Array" {
                println!("SECURITY ALERT: Capability '{}' blocked for '{}'. Execution Halted.", service, target);
                std::process::exit(1);
            }

            match target {
                // ... (IO, Array, Sys rimangono identici - Copia e incolla dal vecchio file) ...
                "IO.print" => {
                    let output: Vec<String> = args.iter().map(|a| self.eval_expression(a).to_string()).collect();
                    println!("{}", output.join(" "));
                    let _ = io::stdout().flush();
                    return Value::Null;
                },
                "IO.input" => {
                    if let Some(prompt_expr) = args.get(0) {
                        let raw_prompt = self.eval_expression(prompt_expr);
                        let p = self.resolve_value(raw_prompt);
                        print!("{}", p);
                        io::stdout().flush().unwrap();
                    }
                    let mut buffer = String::new();
                    io::stdin().read_line(&mut buffer).unwrap();
                    return Value::String(buffer.trim().to_string());
                },
                "Array.len" => {
                    if let Some(arg) = args.get(0) {
                        if let Value::Array(arr) = self.eval_expression(arg) { return Value::Integer(arr.len() as i64); }
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
                        let mut found = false;
                        for scope in self.scopes.iter().rev() {
                            if let Some(entry) = scope.get(var_name) {
                                println!("[RAM DUMP] Variable '{}' -> {:?}", var_name, entry.value);
                                found = true;
                                break;
                            }
                        }
                        if !found { println!("[RAM DUMP] Variable '{}' -> <Not Found>", var_name); }
                    }
                    return Value::Null;
                },
                "FS.read" => {
                    if let Some(path_expr) = args.get(0) {
                        let raw_path = self.eval_expression(path_expr);
                        let path_val = self.resolve_value(raw_path);
                        let path_str = path_val.to_string();
                        let allowed = self.fs_allow_list.iter().any(|prefix| path_str.starts_with(prefix));
                        if !allowed {
                            println!("FS SECURITY ALERT: Access denied to '{}'. Allowed: {:?}", path_str, self.fs_allow_list);
                            return Value::Null;
                        }
                        return fs::read_file(&path_str);
                    }
                    return Value::Null;
                },
                "FS.write" => {
                    if args.len() >= 2 {
                        let raw_path = self.eval_expression(&args[0]);
                        let path_val = self.resolve_value(raw_path);
                        let raw_content = self.eval_expression(&args[1]);
                        let content_val = self.resolve_value(raw_content);
                        let path_str = path_val.to_string();
                        let allowed = self.fs_allow_list.iter().any(|prefix| path_str.starts_with(prefix));
                        if !allowed {
                            println!("FS SECURITY ALERT: Write denied to '{}'. Allowed: {:?}", path_str, self.fs_allow_list);
                            return Value::Boolean(false);
                        }
                        return fs::write_file(&path_str, &content_val.to_string());
                    }
                    return Value::Boolean(false);
                },

                // === NET MODULE (NUOVO) ===
                "Net.get" => {
                    if let Some(url_expr) = args.get(0) {
                        let raw_url = self.eval_expression(url_expr);
                        let url_val = self.resolve_value(raw_url);
                        let url_str = url_val.to_string();

                        // CHECK WHITELIST
                        let allowed = self.net_allow_list.iter().any(|prefix| url_str.starts_with(prefix));
                        if !allowed {
                            println!("NET SECURITY ALERT: Call blocked to '{}'. Allowed: {:?}", url_str, self.net_allow_list);
                            return Value::Null;
                        }
                        
                        return net::get(&url_str);
                    }
                    return Value::Null;
                },
                "Net.post" => {
                    if args.len() >= 2 {
                        let raw_url = self.eval_expression(&args[0]);
                        let url_val = self.resolve_value(raw_url);
                        let raw_body = self.eval_expression(&args[1]);
                        let body_val = self.resolve_value(raw_body);
                        let url_str = url_val.to_string();

                        // CHECK WHITELIST
                        let allowed = self.net_allow_list.iter().any(|prefix| url_str.starts_with(prefix));
                        if !allowed {
                            println!("NET SECURITY ALERT: Call blocked to '{}'. Allowed: {:?}", url_str, self.net_allow_list);
                            return Value::Null;
                        }

                        return net::post(&url_str, &body_val.to_string());
                    }
                    return Value::Null;
                },

                _ => { println!("RUNTIME WARNING: Unknown function target '{}'", target); }
            }
        }

        if let Some(func_stmt) = self.functions.get(target).cloned() {
            if let Statement::FunctionDecl { params, body, .. } = func_stmt {
                let mut new_scope = std::collections::HashMap::new();
                for (i, param_name) in params.iter().enumerate() {
                    let arg_val = if i < args.len() { self.eval_expression(&args[i]) } else { Value::Null };
                    let entry = VarEntry { value: arg_val, is_mutable: true, is_secure: false };
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