use crate::engine::ast::Expression;
use crate::engine::value::Value;
use super::Interpreter;

impl Interpreter {
    pub fn eval_expression(&mut self, expr: &Expression) -> Value {
        match expr {
            Expression::LiteralStr(s) => Value::String(s.clone()),
            Expression::LiteralNum(n) => {
                if n.fract() == 0.0 {
                    Value::Integer(*n as i64)
                } else {
                    Value::Float(*n)
                }
            }
            Expression::LiteralBool(b) => Value::Boolean(*b),
            Expression::LiteralNull => Value::Null,
            Expression::Array(elements) => {
                let vals: Vec<Value> = elements.iter().map(|e| self.eval_expression(e)).collect();
                Value::Array(vals)
            }
            Expression::Map(pairs) => {
                let mut map = std::collections::HashMap::new();
                for (k, v_expr) in pairs {
                    let val = self.eval_expression(v_expr);
                    map.insert(k.clone(), val);
                }
                Value::Map(map)
            }
            Expression::Variable(name) => {
                let val = self.get_var(name);
                if val == Value::Null && self.functions.contains_key(name) {
                    Value::String(name.clone())
                } else {
                    val
                }
            }
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
                        if idx < arr.len() {
                            return arr[idx].clone();
                        }
                    }
                    Value::Map(map) => {
                        let key = index_val.to_string();
                        if let Some(val) = map.get(&key) {
                            return val.clone();
                        }
                    }
                    _ => {}
                }
                Value::Null
            }
            Expression::BinaryOp { left, operator, right } => {
                let l = self.eval_expression(left);
                let r = self.eval_expression(right);
                self.eval_binary_op(l, operator, r)
            }
            Expression::UnaryOp { operator, operand } => {
                let val = self.eval_expression(operand);
                match operator.as_str() {
                    "!" => Value::Boolean(!val.is_truthy()),
                    "-" => match val {
                        Value::Integer(i) => Value::Integer(-i),
                        Value::Float(f) => Value::Float(-f),
                        _ => Value::Null,
                    },
                    _ => Value::Null,
                }
            }
            Expression::Ternary { condition, true_expr, false_expr } => {
                let cond_val = self.eval_expression(condition);
                if cond_val.is_truthy() {
                    self.eval_expression(true_expr)
                } else {
                    self.eval_expression(false_expr)
                }
            }
            Expression::FunctionCall { target, args } => {
                let mut resolved_name = None;

                if let Expression::Variable(ref name) = **target {
                    let is_builtin = matches!(
                        name.as_str(),
                        "print" | "input" | "read" | "write" | "delete" | "sin" | "cos" | "sqrt" | "random" | "round" | "min" | "max" | "abs" | "log" | "pow" | "len" | "sleep" | "exit" | "fetch" | "send"
                    );
                    let has_direct = is_builtin || self.functions.contains_key(name);

                    if has_direct {
                        // Se la configurazione (numero di parametri) è valida per il nome diretto, usalo
                        if self.is_function_arity_valid(name, args.len()) {
                            resolved_name = Some(name.clone());
                        } else {
                            // Altrimenti, cerca se c'è una variabile con lo stesso nome che contiene una stringa
                            let var_val = self.get_var(name);
                            if let Value::String(ref s) = var_val {
                                // E controlla se questa seconda funzione è valida per il numero di parametri
                                if self.is_function_arity_valid(s, args.len()) {
                                    resolved_name = Some(s.clone());
                                }
                            }
                        }

                        // Fallback se nessuna delle due ha una configurazione valida (lasciamo che fallisca sulla diretta per errore di arità)
                        if resolved_name.is_none() {
                            resolved_name = Some(name.clone());
                        }
                    }
                }

                let func_name = match resolved_name {
                    Some(name) => name,
                    None => {
                        let target_val = self.eval_expression(target);
                        match target_val {
                            Value::String(s) => s,
                            _ => {
                                if let Expression::Variable(ref name) = **target {
                                    name.clone()
                                } else {
                                    target_val.to_string()
                                }
                            }
                        }
                    }
                };
                self.handle_function_call(&func_name, args)
            }
        }
    }
}