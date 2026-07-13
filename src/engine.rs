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

use self::lexer::Lexer;
use self::parser::Parser;
use self::interpreter::Interpreter;
use self::translate::TranslationEngine;
use self::import::ImportManager;

pub struct Engine {
    pub translation_engine: TranslationEngine,
    pub import_manager: ImportManager,
    pub interpreter: Interpreter,
    pub quiet: bool,
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
            quiet: false,
        }
    }

    /// Avvia la pipeline completa di NodeStract per un sorgente fornito.
    /// Restituisce `true` se l'esecuzione è terminata senza errori, `false` altrimenti.
    pub fn run(&mut self, source: &str) -> bool {
        // 1. Estrae e valida gli import (riga per riga)
        let (stripped_source, active_import_manager) = match check::validate_imports(source, &self.translation_engine) {
            Ok(res) => res,
            Err(err_msg) => {
                if !self.quiet {
                    crate::welcome::show_error(&err_msg);
                }
                return false;
            }
        };
        self.import_manager = active_import_manager;

        // 2. Costruisce il vocabolario di keyword attive (FilteredEngine)
        let filtered_engine = filter::FilteredEngine::new(&self.translation_engine, &self.import_manager);

        // 3. Tokenizza il sorgente ripulito dagli import
        let mut lexer = Lexer::new(&stripped_source);
        let final_tokens = lexer.tokenize(&self.translation_engine, &filtered_engine);

        // 4. Esegue il parsing e la validazione sintattica dei delimitatori
        let mut parser = Parser::new(final_tokens.clone());
        match parser.parse(&self.translation_engine, &self.import_manager) {
            Ok(program) => {
                self.interpreter = Interpreter::new();
                self.interpreter.run(program);
                if let Some(ref exc) = self.interpreter.exception {
                    if !self.quiet {
                        let exc_str = match exc {
                            crate::engine::value::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        crate::welcome::show_error(&format!("Uncaught Exception: {}", exc_str));
                    }
                    return false;
                }
                true
            }
            Err(err_msg) => {
                if !self.quiet {
                    crate::welcome::show_error(&err_msg);
                }
                false
            }
        }
    }

    /// Legge un file da disco e lo esegue nella pipeline.
    /// Il messaggio di successo viene mostrato solo se l'esecuzione non ha prodotto errori.
    pub fn run_file(&mut self, filename: &str) {
        match std::fs::read_to_string(filename) {
            Ok(content) => {
                if self.run(&content) {
                    crate::welcome::show_success("Execution finished successfully.");
                }
            }
            Err(_) => {
                crate::welcome::show_error(&format!("Could not read file '{}'. Check the path.", filename));
            }
        }
    }
}
