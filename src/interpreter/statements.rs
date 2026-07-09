use crate::engine::ast::{Statement, Expression};
use crate::engine::value::Value;
use super::Interpreter;

impl Interpreter {
    pub fn execute_statement(&mut self, stmt: &Statement) {
        if self.last_return.is_some() || self.loop_break || self.loop_continue || self.exception.is_some() {
            return;
        }

        match stmt {
            Statement::VarDecl { name, value, is_mutable } => {
                let val = self.eval_expression(value);
                self.define_var(name.clone(), val, *is_mutable);
            }
            Statement::Assignment { target, value } => {
                let val = self.eval_expression(value);
                
                let mut path = Vec::new();
                let mut current = target;
                while let Expression::Index { target: inner_target, index } = current {
                    let idx_val = self.eval_expression(index);
                    path.push(idx_val);
                    current = inner_target;
                }
                
                let var_name = match current {
                    Expression::Variable(name) => name,
                    _ => {
                        let err_msg = "Target di assegnazione non valido.".to_string();
                        self.exception = Some(Value::String(err_msg));
                        return;
                    }
                };
                
                path.reverse();
                
                if let Some(entry) = self.get_var_mut(var_name) {
                    if !entry.is_mutable {
                        let err_msg = format!("Impossibile assegnare a una costante '{}'.", var_name);
                        self.exception = Some(Value::String(err_msg));
                        return;
                    }
                    if let Err(err) = Self::mutate_value_at_path(&mut entry.value, &path, val) {
                        self.exception = Some(Value::String(err));
                    }
                } else {
                    let err_msg = format!("Variabile '{}' non dichiarata prima dell'assegnazione.", var_name);
                    self.exception = Some(Value::String(err_msg));
                }
            }
            Statement::IfStatement { condition, then_branch, else_branch } => {
                let raw_cond = self.eval_expression(condition);
                if raw_cond.is_truthy() {
                    self.enter_scope();
                    for s in then_branch {
                        self.execute_statement(s);
                        if self.last_return.is_some() || self.loop_break || self.loop_continue || self.exception.is_some() {
                            break;
                        }
                    }
                    self.exit_scope();
                } else if let Some(else_stmts) = else_branch {
                    self.enter_scope();
                    for s in else_stmts {
                        self.execute_statement(s);
                        if self.last_return.is_some() || self.loop_break || self.loop_continue || self.exception.is_some() {
                            break;
                        }
                    }
                    self.exit_scope();
                }
            }
            Statement::WhileStatement { condition, body } => {
                loop {
                    let raw_cond = self.eval_expression(condition);
                    if !raw_cond.is_truthy() {
                        break;
                    }

                    self.enter_scope();
                    for s in body {
                        self.execute_statement(s);
                        if self.last_return.is_some() || self.exception.is_some() {
                            self.exit_scope();
                            return;
                        }
                        if self.loop_break {
                            self.exit_scope();
                            self.loop_break = false;
                            return;
                        }
                        if self.loop_continue {
                            self.loop_continue = false;
                            break;
                        }
                    }
                    self.exit_scope();
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
                    self.enter_scope();
                    self.define_var(iterator.clone(), Value::Integer(i), false);
                    for s in body {
                        self.execute_statement(s);
                        if self.last_return.is_some() || self.exception.is_some() {
                            self.exit_scope();
                            return;
                        }
                        if self.loop_break {
                            self.exit_scope();
                            self.loop_break = false;
                            return;
                        }
                        if self.loop_continue {
                            self.loop_continue = false;
                            break;
                        }
                    }
                    self.exit_scope();
                }
            }
            Statement::SwitchStatement { discriminant, cases, default_case } => {
                let disc_val = self.eval_expression(discriminant);
                let mut matched = false;
                for (test_expr, body) in cases {
                    let test_val = self.eval_expression(test_expr);
                    if disc_val == test_val {
                        matched = true;
                        self.enter_scope();
                        for s in body {
                            self.execute_statement(s);
                            if self.last_return.is_some() || self.loop_break || self.loop_continue || self.exception.is_some() {
                                break;
                            }
                        }
                        self.exit_scope();
                        break;
                    }
                }
                if !matched {
                    if let Some(body) = default_case {
                        self.enter_scope();
                        for s in body {
                            self.execute_statement(s);
                            if self.last_return.is_some() || self.loop_break || self.loop_continue || self.exception.is_some() {
                                break;
                            }
                        }
                        self.exit_scope();
                    }
                }
            }
            Statement::TryCatchStatement { try_block, catch_variable, catch_block, finally_block } => {
                self.enter_scope();
                for s in try_block {
                    self.execute_statement(s);
                    if self.exception.is_some() || self.last_return.is_some() || self.loop_break || self.loop_continue {
                        break;
                    }
                }
                self.exit_scope();

                if self.exception.is_some() {
                    if let Some(ref catch_stmts) = catch_block {
                        let exc = self.exception.take().unwrap();
                        self.enter_scope();
                        if let Some(ref var_name) = catch_variable {
                            self.define_var(var_name.clone(), exc, false);
                        }
                        for s in catch_stmts {
                            self.execute_statement(s);
                            if self.last_return.is_some() || self.loop_break || self.loop_continue || self.exception.is_some() {
                                break;
                            }
                        }
                        self.exit_scope();
                    }
                }

                if let Some(finally_stmts) = finally_block {
                    let saved_return = self.last_return.take();
                    let saved_break = self.loop_break;
                    let saved_continue = self.loop_continue;
                    let saved_exception = self.exception.take();

                    self.loop_break = false;
                    self.loop_continue = false;

                    self.enter_scope();
                    for s in finally_stmts {
                        self.execute_statement(s);
                    }
                    self.exit_scope();

                    if self.last_return.is_none() && !self.loop_break && !self.loop_continue && self.exception.is_none() {
                        self.last_return = saved_return;
                        self.loop_break = saved_break;
                        self.loop_continue = saved_continue;
                        self.exception = saved_exception;
                    }
                }
            }
            Statement::ThrowStatement { value } => {
                self.exception = Some(self.eval_expression(value));
            }
            Statement::ReturnStatement { value } => {
                self.last_return = Some(self.eval_expression(value));
            }
            Statement::Break => {
                self.loop_break = true;
            }
            Statement::Continue => {
                self.loop_continue = true;
            }
            Statement::FunctionDecl { name, .. } => {
                self.functions.insert(name.clone(), stmt.clone());
            }
            Statement::Expr(expr) => {
                self.eval_expression(expr);
            }
        }
    }
}