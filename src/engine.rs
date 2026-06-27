pub mod lexer;
pub mod ast;
pub mod parser;
pub mod interpreter;
pub mod value;
pub mod translate;
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
        println!("[Engine] [1/4] Starting Lexer...");
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize(); 
        println!("[Engine] Lexer completed. Generated {} tokens.", tokens.len());

        println!("[Engine] [2/4] Starting Parser...");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        println!("[Engine] Parser completed. AST successfully generated.");

        println!("[Engine] [3/4] Processing imports & validation...");
        // In the next phase, we'll walk the AST to extract and register active imports in import_manager

        println!("[Engine] [4/4] Starting Execution...");
        println!("--------------------------------------------------");
        self.interpreter.run(ast);
        println!("--------------------------------------------------");
        println!("[Engine] Execution finished.");
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
