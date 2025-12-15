use crate::ast::Statement;
use crate::value::Value;
use super::Interpreter;

impl Interpreter {
    pub fn execute_statement(&mut self, stmt: &Statement) {
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
                let raw_cond = self.eval_expression(condition);
                let cond_val = self.resolve_value(raw_cond);

                if cond_val.is_truthy() {
                    self.enter_scope();
                    for s in then_branch { self.execute_statement(s); }
                    self.exit_scope();
                } else if let Some(else_stmts) = else_branch {
                    self.enter_scope();
                    for s in else_stmts { self.execute_statement(s); }
                    self.exit_scope();
                }
            }
            Statement::WhileStatement { condition, body } => {
                loop {
                    let raw_cond = self.eval_expression(condition);
                    let cond_val = self.resolve_value(raw_cond);
                    if !cond_val.is_truthy() { break; }

                    self.enter_scope();
                    for s in body {
                        self.execute_statement(s);
                        if self.last_return.is_some() { self.exit_scope(); return; }
                    }
                    self.exit_scope();
                }
            }
            Statement::ForStatement { iterator, start, end, body } => {
                let raw_start = self.eval_expression(start);
                let start_val = self.resolve_value(raw_start);
                let raw_end = self.eval_expression(end);
                let end_val = self.resolve_value(raw_end);

                let start_int = match start_val { Value::Integer(i) => i, Value::Float(f) => f as i64, _ => 0 };
                let end_int = match end_val { Value::Integer(i) => i, Value::Float(f) => f as i64, _ => 0 };

                for i in start_int..end_int {
                    self.enter_scope();
                    self.define_var(iterator.clone(), Value::Integer(i), false, false);
                    for s in body {
                        self.execute_statement(s);
                        if self.last_return.is_some() { self.exit_scope(); return; }
                    }
                    self.exit_scope();
                }
            }
            Statement::ReturnStatement { value } => {
                self.last_return = Some(self.eval_expression(value));
            }
            Statement::Expr(expr) => { self.eval_expression(expr); }
            _ => {}
        }
    }
}