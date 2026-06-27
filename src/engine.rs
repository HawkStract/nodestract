#[path = "lexer/lexer.rs"]
pub mod lexer;
#[path = "ast.rs"]
pub mod ast;
#[path = "parser.rs"]
pub mod parser;
#[path = "interpreter.rs"]
pub mod interpreter;
#[path = "value.rs"]
pub mod value;
#[path = "translate/translate.rs"]
pub mod translate;
#[path = "import/import.rs"]
pub mod import;

use self::lexer::Lexer;
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
    /// Create a new Engine instance with the target language for keyword translation.
    pub fn new(lang: &str) -> Self {
        println!("[Engine] Initializing Translation Engine for language: '{}'...", lang);
        let translation_engine = TranslationEngine::new(lang);
        
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
        println!("[Engine] [1/2] Starting Lexer...");
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize(&self.translation_engine); 
        println!("[Engine] Lexer completed. Generated {} tokens.", tokens.len());

        println!("[Engine] [2/2] Displaying Tokens:");
        let token_reprs: Vec<String> = tokens
            .iter()
            .map(|t| t.to_string_repr())
            .collect();
        
        println!("--------------------------------------------------");
        println!("[ {} ]", token_reprs.join(" | "));
        println!("--------------------------------------------------");
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
