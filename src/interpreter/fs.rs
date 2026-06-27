use std::fs;
use std::path::Path;
use crate::engine::value::Value;

pub fn read_file(path: &str) -> Value {
    if !path.ends_with(".nso") && !path.ends_with(".txt") {
        println!("FS Error: Only .nso or .txt files allowed.");
        return Value::Null;
    }
    match fs::read_to_string(path) {
        Ok(content) => Value::String(content),
        Err(_) => Value::Null,
    }
}

pub fn write_file(path: &str, content: &str) -> Value {
    if !path.ends_with(".nso") && !path.ends_with(".txt") {
        println!("FS Error: Only .nso or .txt files allowed.");
        return Value::Boolean(false);
    }
    
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }

    match fs::write(path, content) {
        Ok(_) => Value::Boolean(true),
        Err(_) => Value::Boolean(false),
    }
}