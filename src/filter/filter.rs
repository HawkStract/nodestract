use std::collections::HashMap;
use crate::engine::translate::TranslationEngine;
use crate::engine::import::ImportManager;

pub struct FilteredEngine {
    // Mappa le parole normalizzate in minuscolo alle loro forme canoniche inglesi
    active_keywords: HashMap<String, String>,
}

impl FilteredEngine {
    /// Crea un FilteredEngine con solo le keyword autorizzate dagli import attivi.
    pub fn new(translation: &TranslationEngine, import_manager: &ImportManager) -> Self {
        let mut active_keywords = HashMap::new();

        for (normalized_word, candidates) in &translation.keyword_map {
            for (canonical, module, language) in candidates {
                // Disabilita le keyword di bootstrap ed i nomi delle lingue
                if canonical == "import"
                    || canonical == "from"
                    || canonical == "english"
                    || canonical == "italian"
                    || canonical == "spanish"
                    || canonical == "french"
                    || canonical == "german"
                    || canonical == "portuguese"
                    || canonical == "romanian"
                {
                    continue;
                }

                // Conserva la keyword se la lingua di appartenenza è attiva
                if language == "english"
                    || language == "italian"
                    || language == "spanish"
                    || language == "french"
                    || language == "german"
                    || language == "portuguese"
                    || language == "romanian"
                {
                    if import_manager.is_member_active(language, "translate") {
                        active_keywords.insert(normalized_word.clone(), canonical.clone());
                    }
                }
                // Conserva il built-in se il rispettivo modulo è attivo
                else if module == "nio"
                    || module == "nmath"
                    || module == "nfs"
                    || module == "nnet"
                {
                    if import_manager.is_member_active(canonical, module) {
                        active_keywords.insert(normalized_word.clone(), canonical.clone());
                    }
                }
            }
        }

        Self { active_keywords }
    }

    /// Cerca una parola nel set di keyword attive.
    pub fn lookup(&self, word: &str, translation: &TranslationEngine) -> Option<&str> {
        let normalized = translation.normalize(word);
        self.active_keywords.get(&normalized).map(|s| s.as_str())
    }
}
