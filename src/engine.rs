#![allow(dead_code, unused_imports)]
#[path = "lexer/lexer.rs"]
pub mod lexer;
#[path = "ast/ast.rs"]
pub mod ast;
#[path = "parser/parser.rs"]
pub mod parser;
#[path = "interpreter/interpreter.rs"]
pub mod interpreter;
#[path = "value.rs"]
pub mod value;
#[path = "translate/translate.rs"]
pub mod translate;
#[path = "import/import.rs"]
pub mod import;
#[path = "import/check.rs"]
pub mod check;
#[path = "filter/filter.rs"]
pub mod filter;

use self::lexer::{Lexer, Token};
use self::parser::Parser;
use self::interpreter::Interpreter;
use self::translate::TranslationEngine;
use self::import::ImportManager;

pub struct Engine {
    pub translation_engine: TranslationEngine,
    pub import_manager: ImportManager,
    pub interpreter: Interpreter,
}

impl Engine {
    /// Create a new Engine instance.
    pub fn new() -> Self {
        println!("[Engine] Initializing Translation Engine (supporting all active languages)...");
        let translation_engine = TranslationEngine::new();
        
        println!("[Engine] Initializing Import Manager...");
        let import_manager = ImportManager::new();

        println!("[Engine] Initializing Runtime Interpreter...");
        let interpreter = Interpreter::new();

        Self {
            translation_engine,
            import_manager,
            interpreter,
        }
    }

    /// Runs the complete NodeStract pipeline for a given source code.
    pub fn run(&mut self, source: &str) {
        // 1. Extract and validate imports (line-by-line)
        println!("[Engine] [1/3] Validating and extracting import header...");
        let (stripped_source, active_import_manager) = match check::validate_imports(source, &self.translation_engine) {
            Ok(res) => res,
            Err(err_msg) => {
                crate::welcome::show_error(&err_msg);
                println!("[Engine] Execution aborted due to import validation errors.");
                return;
            }
        };
        self.import_manager = active_import_manager;

        // 2. Build filtered engine (active keywords vocabulary)
        println!("[Engine] [2/3] Filtering active vocabulary for tokenization...");
        let filtered_engine = filter::FilteredEngine::new(&self.translation_engine, &self.import_manager);

        // 3. Tokenize the stripped source code
        let mut lexer = Lexer::new(&stripped_source);
        let final_tokens = lexer.tokenize(&self.translation_engine, &filtered_engine);

        // Verify that any built-in functions referenced in the code are actually imported
        for token in &final_tokens {
            if let Token::Identifier(ref name) = token {
                if let Some((canonical, module)) = self.translation_engine.get_builtin_info(name) {
                    if !self.import_manager.is_member_active(canonical, module) {
                        crate::welcome::show_error(&format!(
                            "Import Error: Built-in function '{}' used but its library module '{}' was not imported",
                            name, module
                        ));
                        println!("[Engine] Execution aborted due to missing imports.");
                        return;
                    }
                }
            }
        }

        // 4. Run Parser for structural and delimiter validation
        println!("[Engine] [3/3] Starting Parser (Syntax Validation)...");
        let mut parser = Parser::new(final_tokens.clone());
        match parser.parse() {
            Ok(program) => {
                println!("[Engine] Syntax and imports are flawless! Displaying Tokens (stripped header):");
                let token_reprs: Vec<String> = final_tokens
                    .iter()
                    .map(|t| t.to_string_repr())
                    .collect();
                
                println!("--------------------------------------------------");
                println!("[ {} ]", token_reprs.join(" | "));
                println!("--------------------------------------------------");

                println!("[Engine] Starting Runtime Interpreter...");
                self.interpreter = Interpreter::new();
                self.interpreter.run(program);
            }
            Err(err_msg) => {
                crate::welcome::show_error(&format!("Syntax Error: {}", err_msg));
                println!("[Engine] Execution aborted due to syntax errors.");
            }
        }
    }

    /// Reads a file from disk and runs the compiler pipeline.
    pub fn run_file(&mut self, filename: &str) {
        println!("[Engine] Loading file: {}", filename);
        match std::fs::read_to_string(filename) {
            Ok(content) => {
                self.run(&content);
                crate::welcome::show_success("Execution finished successfully.");
            }
            Err(_) => {
                crate::welcome::show_error(&format!("Could not read file '{}'. Check the path.", filename));
            }
        }
    }
}
