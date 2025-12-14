mod lexer;
mod ast;
mod parser;
mod interpreter;
mod value;

use std::env;
use std::process;
use std::fs;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::interpreter::Interpreter;

const COLOR_RESET: &str = "\x1b[0m";
const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_CYAN: &str = "\x1b[36m";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_banner();
        print_usage();
        process::exit(0);
    }

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
            println!("Node Stract Compiler (NSC) v0.6.0 - HawkStract Ecosystem");
        },
        _ => {
            println!("{}Unknown command: '{}'{}", COLOR_RED, command, COLOR_RESET);
            print_usage();
        }
    }
}

fn print_banner() {
    println!("{}
    _   _           _        ____  _                  _   
   | \\ | | ___  ___| | ___  / ___|| |_ _ __ __ _  ___| |_ 
   |  \\| |/ _ \\/  _` |/ _ \\ \\___ \\| __| '__/ _` |/ __| __|
   | |\\  | (_) \\ (_| |  __/  ___) | |_| | | (_| | (__| |_ 
   |_| \\_|\\___/ \\__,_|\\___| |____/ \\__|_|  \\__,_|\\___|\\__|
   
   HawkStract Ecosystem - Secure. Atomic. Abstract.
   {}", COLOR_CYAN, COLOR_RESET);
}

fn print_usage() {
    println!("Usage:");
    println!("  nsc build <file.hns>   Compile a Node Stract file");
    println!("  nsc version            Show version info");
}

fn cmd_build(filename: &str) {
    println!("{}---> Starting build process for: {}{}", COLOR_GREEN, filename, COLOR_RESET);
    
    match fs::read_to_string(filename) {
        Ok(content) => {
            println!("     File found. Size: {} bytes", content.len());
            
            println!("     [1/3] Lexing phase...");
            let mut lexer = Lexer::new(&content);
            let tokens = lexer.tokenize();
            println!("          Generated {} tokens.", tokens.len());

            println!("     [2/3] Parsing phase...");
            let mut parser = Parser::new(tokens);
            let ast = parser.parse();
            
            println!("     [3/3] Executing (Interpreter Mode)...");
            println!("--------------------------------------------------");
            let mut interpreter = Interpreter::new();
            interpreter.run(ast);
            println!("--------------------------------------------------");

            println!("{}---> Execution Successful{}", COLOR_GREEN, COLOR_RESET);
        },
        Err(_) => {
            println!("{}Error: Could not read file '{}'. Check the path.{}", COLOR_RED, filename, COLOR_RESET);
        }
    }
}