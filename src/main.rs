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
                welcome::show_error("Missing filename. Usage: nsc build <filename.ns>");
                return;
            }
            let filename = &args[2];

            let mut engine = Engine::new();
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