use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

static RETRY_COUNT: AtomicU32 = AtomicU32::new(0);

fn start_mock_server() {
    thread::spawn(|| {
        let listener = match TcpListener::bind("127.0.0.1:12345") {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[Mock Server] Failed to bind: {}", e);
                return;
            }
        };
        
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buffer = [0; 1024];
                if stream.read(&mut buffer).is_ok() {
                    let req = String::from_utf8_lossy(&buffer);
                    let response;
                    
                    if req.starts_with("GET /success") {
                        let body = "Hello NodeStract";
                        response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
                            body.len(),
                            body
                        );
                    } else if req.starts_with("GET /notfound") {
                        response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_string();
                    } else if req.starts_with("POST /post") {
                        let body = "POST received";
                        response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
                            body.len(),
                            body
                        );
                    } else if req.starts_with("GET /retry") {
                        let attempts = RETRY_COUNT.fetch_add(1, Ordering::SeqCst);
                        if attempts == 0 {
                            response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n".to_string();
                        } else {
                            let body = "Retry success";
                            response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
                                body.len(),
                                body
                            );
                        }
                    } else {
                        response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n".to_string();
                    }
                    
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();
                }
            }
        }
    });
}

fn run_test_file(path: &str) -> Result<String, String> {
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "build", path])
        .output()
        .map_err(|e| format!("Failed to execute cargo run: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let full_output = format!("{}\n{}", stdout, stderr);

    if !output.status.success() {
        return Err(format!(
            "Process exited with non-zero status.\n--- OUTPUT ---\n{}\n--------------",
            full_output
        ));
    }

    if full_output.contains("FAIL") {
        return Err(format!(
            "Test output contains 'FAIL'.\n--- OUTPUT ---\n{}\n--------------",
            full_output
        ));
    }

    Ok(stdout)
}

fn main() {
    println!("=== AVVIO DI TEST SUITE PER NODESTRACT ===");
    
    // Start local server for network tests
    start_mock_server();
    // Wait briefly for the listener to spin up
    thread::sleep(Duration::from_millis(100));

    let categories = &["languages", "typing", "data", "net", "function", "conditional", "logical", "math"];
    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut failures = Vec::new();

    for category in categories {
        let dir_path = format!("examples/test/{}", category);
        println!("\n[Categoria: {}]", category);

        let mut paths = Vec::new();
        if let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "ns") {
                        paths.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        // Sort paths alphabetically
        paths.sort();

        if paths.is_empty() {
            println!("  Nessun test trovato.");
            continue;
        }

        for path in paths {
            print!("  Esecuzione {} ... ", path);
            std::io::stdout().flush().unwrap();
            
            match run_test_file(&path) {
                Ok(_) => {
                    println!("OK");
                    total_passed += 1;
                }
                Err(err_msg) => {
                    println!("FALLITO");
                    total_failed += 1;
                    failures.push((path.clone(), err_msg));
                }
            }
        }
    }

    println!("\n=== RIEPILOGO TEST ===");
    println!("Superati: {}", total_passed);
    println!("Falliti:  {}", total_failed);

    if !failures.is_empty() {
        println!("\n=== DETTAGLI FALLIMENTI ===");
        for (path, err) in &failures {
            println!("\n[!] Test fallito: {}\n{}", path, err);
        }
        std::process::exit(1);
    } else {
        println!("\n[Tutti i test sono stati superati con successo!]");
        std::process::exit(0);
    }
}
