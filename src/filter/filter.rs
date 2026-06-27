use std::collections::HashMap;
use crate::engine::translate::TranslationEngine;
use crate::engine::import::ImportManager;

pub struct FilteredEngine {
    // Maps lowercase, normalized words to their canonical English forms
    active_keywords: HashMap<String, String>,
}

impl FilteredEngine {
    /// Builds a new FilteredEngine containing only keywords authorized by active imports.
    /// It automatically disables the import/from keywords and language names.
    pub fn new(translation: &TranslationEngine, import_manager: &ImportManager) -> Self {
        let mut active_keywords = HashMap::new();

        for (normalized_word, candidates) in &translation.keyword_map {
            for (canonical, module, language) in candidates {
                // 1. Automatically disable core bootstrap keywords and language names
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

                // 2. Keep keyword if its language module is active
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
                // 3. Keep built-in if its library module is active
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

    /// Looks up a word in the active/authorized keywords set.
    pub fn lookup(&self, word: &str, translation: &TranslationEngine) -> Option<&str> {
        let normalized = translation.normalize(word);
        self.active_keywords.get(&normalized).map(|s| s.as_str())
    }
}
