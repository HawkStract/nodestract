mod engine;
mod welcome;

use std::env;
use crate::engine::Engine;

fn main() {
    let args: Vec<String> = env::args().collect();

    // If no arguments are provided, show welcome screen
    if args.len() < 2 {
        welcome::show_welcome();
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "build" => {
            if args.len() < 3 {
                welcome::show_error("Missing filename. Usage: nsc build <filename.ns> [--lang <lang>]");
                return;
            }
            let filename = &args[2];

            // Parse optional --lang parameter (defaults to English "en")
            let lang = args.iter()
                .position(|arg| arg == "--lang")
                .and_then(|idx| args.get(idx + 1))
                .map(|s| s.as_str())
                .unwrap_or("en");

            let mut engine = Engine::new(lang);
            engine.run_file(filename);
        },
        "version" => {
            welcome::show_version();
        },
        _ => {
            welcome::show_error(&format!("Unknown command: '{}'", command));
            welcome::show_usage();
        }
    }
}