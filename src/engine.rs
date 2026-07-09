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
    /// Inizializza una nuova istanza dell'Engine.
    pub fn new() -> Self {
        let translation_engine = TranslationEngine::new();
        let import_manager = ImportManager::new();
        let interpreter = Interpreter::new();
        Self {
            translation_engine,
            import_manager,
            interpreter,
        }
    }

    /// Avvia la pipeline completa di NodeStract per un sorgente fornito.
    pub fn run(&mut self, source: &str) {
        // 1. Estrae e valida gli import (riga per riga)
        let (stripped_source, active_import_manager) = match check::validate_imports(source, &self.translation_engine) {
            Ok(res) => res,
            Err(err_msg) => {
                crate::welcome::show_error(&err_msg);
                return;
            }
        };
        self.import_manager = active_import_manager;

        // 2. Costruisce il vocabolario di keyword attive (FilteredEngine)
        let filtered_engine = filter::FilteredEngine::new(&self.translation_engine, &self.import_manager);

        // 3. Tokenizza il sorgente ripulito dagli import
        let mut lexer = Lexer::new(&stripped_source);
        let final_tokens = lexer.tokenize(&self.translation_engine, &filtered_engine);

        // Verifica che le funzioni di sistema usate siano effettivamente importate
        for token in &final_tokens {
            if let Token::Identifier(ref name) = token {
                if let Some((canonical, module)) = self.translation_engine.get_builtin_info(name) {
                    if !self.import_manager.is_member_active(canonical, module) {
                        crate::welcome::show_error(&format!(
                            "Import Error: Built-in function '{}' used but its library module '{}' was not imported",
                            name, module
                        ));
                        return;
                    }
                }
            }
        }

        // 4. Esegue il parsing e la validazione sintattica dei delimitatori
        let mut parser = Parser::new(final_tokens.clone());
        match parser.parse() {
            Ok(program) => {
                self.interpreter = Interpreter::new();
                self.interpreter.run(program);
                if let Some(ref exc) = self.interpreter.exception {
                    crate::welcome::show_error(&format!("Uncaught Exception: {:?}", exc));
                }
            }
            Err(err_msg) => {
                crate::welcome::show_error(&format!("Syntax Error: {}", err_msg));
            }
        }
    }

    /// Legge un file da disco e lo esegue nella pipeline.
    pub fn run_file(&mut self, filename: &str) {
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
