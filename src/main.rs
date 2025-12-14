use std::env;
use std::process;
use std::fs;

// Definiamo i colori per l'output in console (HawkStract Style)
const COLOR_RESET: &str = "\x1b[0m";
const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_CYAN: &str = "\x1b[36m";

fn main() {
    // 1. Legge gli argomenti passati da riga di comando
    let args: Vec<String> = env::args().collect();

    // 2. Se non ci sono argomenti, mostra l'help
    if args.len() < 2 {
        print_banner();
        print_usage();
        process::exit(0);
    }

    // 3. Parsing del comando (es. "build", "version")
    let command = &args[1];

    match command.as_str() {
        "build" => {
            if args.len() < 3 {
                println!("{}Error: Missing filename. Usage: nsc build <filename.hns>{}", COLOR_RED, COLOR_RESET);
                process::exit(1);
            }
            let filename = &args[2];
            cmd_build(filename);
        },
        "version" => {
            println!("NodeStract Compiler (NSC) v0.1.0 - HawkStract Ecosystem");
        },
        _ => {
            println!("{}Unknown command: '{}'{}", COLOR_RED, command, COLOR_RESET);
            print_usage();
        }
    }
}

fn print_banner() {
    println!("{}
    _   _           _      _   _    ____ 
   | \\ | | ___   __| | ___| \\ | |  / ___|
   |  \\| |/ _ \\ / _` |/ _ \\  \\| |  \\___ \\
   | |\\  | (_) | (_| |  __/ |\\  |   ___) |
   |_| \\_|\\___/ \\__,_|\\___|_| \\_|  |____/ 
   
   HawkStract Ecosystem - Secure. Atomic. Abstract.
   {}", COLOR_CYAN, COLOR_RESET);
}

fn print_usage() {
    println!("Usage:");
    println!("  nsc build <file.hns>   Compile a NodeStract file");
    println!("  nsc version            Show version info");
}

// Funzione che simula la compilazione (Core Logic)
fn cmd_build(filename: &str) {
    println!("{}---> Starting build process for: {}{}", COLOR_GREEN, filename, COLOR_RESET);
    
    // Prova a leggere il file
    match fs::read_to_string(filename) {
        Ok(content) => {
            println!("     File found. Size: {} bytes", content.len());
            println!("     [1/3] Lexing phase... DONE");
            println!("     [2/3] Parsing phase... DONE");
            println!("     [3/3] Compiling to Native... DONE");
            println!("{}---> Build Successful (Simulated){}", COLOR_GREEN, COLOR_RESET);
        },
        Err(_) => {
            println!("{}Error: Could not read file '{}'. Check the path.{}", COLOR_RED, filename, COLOR_RESET);
        }
    }
}