use crate::value::Value;
use super::Interpreter;

impl Interpreter {
    pub fn resolve_value(&self, val: Value) -> Value {
        if let Value::String(ref s) = val {
            if s.starts_with("ENC:") {
                let decrypted = Self::decrypt_vault(s);
                if let Ok(i) = decrypted.parse::<i64>() { return Value::Integer(i); }
                if let Ok(f) = decrypted.parse::<f64>() { return Value::Float(f); }
                return Value::String(decrypted);
            }
        }
        val
    }

    pub fn eval_binary_op(&self, left: Value, operator: &str, right: Value) -> Value {
        let l_res = self.resolve_value(left);
        let r_res = self.resolve_value(right);

        match (l_res, r_res) {
            (Value::Integer(a), Value::Integer(b)) => match operator {
                "+" => Value::Integer(a + b), "-" => Value::Integer(a - b),
                "*" => Value::Integer(a * b), "/" => Value::Integer(a / b),
                ">" => Value::Boolean(a > b), "<" => Value::Boolean(a < b),
                ">=" => Value::Boolean(a >= b), "<=" => Value::Boolean(a <= b),
                "==" => Value::Boolean(a == b), "!=" => Value::Boolean(a != b),
                "&&" => Value::Boolean(a != 0 && b != 0), "||" => Value::Boolean(a != 0 || b != 0),
                _ => Value::Null,
            },
            (Value::Float(a), Value::Float(b)) => match operator {
                "+" => Value::Float(a + b), "-" => Value::Float(a - b),
                "*" => Value::Float(a * b), "/" => Value::Float(a / b),
                ">" => Value::Boolean(a > b), "<" => Value::Boolean(a < b),
                ">=" => Value::Boolean(a >= b), "<=" => Value::Boolean(a <= b),
                "==" => Value::Boolean(a == b), "!=" => Value::Boolean(a != b),
                _ => Value::Null,
            },
            (Value::Boolean(a), Value::Boolean(b)) => match operator {
                "&&" => Value::Boolean(a && b), "||" => Value::Boolean(a || b),
                "==" => Value::Boolean(a == b), "!=" => Value::Boolean(a != b),
                _ => Value::Null,
            },
            (Value::Integer(a), Value::Float(b)) => self.eval_binary_op(Value::Float(a as f64), operator, Value::Float(b)),
            (Value::Float(a), Value::Integer(b)) => self.eval_binary_op(Value::Float(a), operator, Value::Float(b as f64)),
            (Value::String(a), Value::String(b)) => match operator {
                "+" => Value::String(a + &b), "==" => Value::Boolean(a == b), "!=" => Value::Boolean(a != b), _ => Value::Null,
            },
            (Value::String(a), b) => match operator { "+" => Value::String(format!("{}{}", a, b)), _ => Value::Null, },
            (a, Value::String(b)) => match operator { "+" => Value::String(format!("{}{}", a, b)), _ => Value::Null, },
            _ => Value::Null,
        }
    }
}