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
            Expression::Variable(name) => self.get_var(name),
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
            Expression::FunctionCall { target, args } => self.handle_function_call(target, args),
            Expression::Await(value) => self.eval_expression(value),
        }
    }
}