use std::collections::HashMap;

pub struct TranslationEngine {
    pub(crate) keyword_map: HashMap<String, Vec<(String, String, String)>>,
    module_map: HashMap<String, String>,
}

impl TranslationEngine {
    /// Crea il motore caricando tutte le lingue supportate.
    pub fn new() -> Self {
        let mut engine = Self {
            keyword_map: HashMap::new(),
            module_map: HashMap::new(),
        };
        engine.load_language("en");
        engine.load_language("it");
        engine.load_language("es");
        engine.load_language("fr");
        engine.load_language("de");
        engine.load_language("pt");
        engine.load_language("ro");
        engine
    }

    fn load_language(&mut self, lang: &str) {
        let json_content = match lang {
            "en" => include_str!("languages/en.json"),
            "it" => include_str!("languages/it.json"),
            "es" => include_str!("languages/es.json"),
            "fr" => include_str!("languages/fr.json"),
            "de" => include_str!("languages/de.json"),
            "pt" => include_str!("languages/pt.json"),
            "ro" => include_str!("languages/ro.json"),
            _ => "",
        };

        if json_content.is_empty() {
            return;
        }

        if let Ok(map) = serde_json::from_str::<HashMap<String, (String, String)>>(json_content) {
            let lang_name = match lang {
                "en" => "english",
                "it" => "italian",
                "es" => "spanish",
                "fr" => "french",
                "de" => "german",
                "pt" => "portuguese",
                "ro" => "romanian",
                other => other,
            }.to_string();

            for (canonical_kw, (translation, module)) in map {
                let normalized = self.normalize(&translation);
                
                // Le keyword di base e i nomi delle lingue non hanno dipendenze (sempre attivi per l'avvio)
                let is_bootstrap = canonical_kw == "import" 
                    || canonical_kw == "from"
                    || canonical_kw == "english"
                    || canonical_kw == "italian"
                    || canonical_kw == "spanish"
                    || canonical_kw == "french"
                    || canonical_kw == "german"
                    || canonical_kw == "portuguese"
                    || canonical_kw == "romanian";

                let dep_lang = if is_bootstrap {
                    "".to_string()
                } else {
                    lang_name.clone()
                };

                let candidates = self.keyword_map.entry(normalized).or_insert_with(Vec::new);
                if !candidates.iter().any(|(c, m, l)| c == &canonical_kw && m == &module && l == &dep_lang) {
                    candidates.push((canonical_kw.clone(), module.clone(), dep_lang));
                }

                self.module_map.insert(canonical_kw, module);
            }
        }
    }

    /// Normalizza una stringa in minuscolo e rimuove gli accenti.
    pub fn normalize(&self, text: &str) -> String {
        let mut normalized = String::new();
        for c in text.to_lowercase().chars() {
            let stripped = match c {
                'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'æ' | 'ă' => 'a',
                'è' | 'é' | 'ê' | 'ë' => 'e',
                'ì' | 'í' | 'î' | 'ï' => 'i',
                'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => 'o',
                'ù' | 'ú' | 'û' | 'ü' => 'u',
                'ç' => 'c',
                'ñ' => 'n',
                'ß' => 's',
                'ș' | 'ş' => 's',
                'ț' | 'ţ' => 't',
                other => other,
            };
            normalized.push(stripped);
        }
        normalized
    }

    /// Risolve una keyword localizzata nella forma inglese canonica se il rispettivo modulo è importato.
    pub fn lookup(&self, word: &str, import_manager: &crate::engine::import::ImportManager) -> Option<&str> {
        let normalized = self.normalize(word);
        if let Some(candidates) = self.keyword_map.get(&normalized) {
            for (canonical, module, language) in candidates {
                let lang_active = language.is_empty() || import_manager.is_member_active(language, "translate");
                if lang_active {
                    if module == "nio" || module == "nmath" || module == "nfs" || module == "nnet" {
                        if import_manager.is_member_active(canonical, module) {
                            return Some(canonical.as_str());
                        }
                    } else {
                        return Some(canonical.as_str());
                    }
                }
            }
        }
        None
    }

    /// Lookup speciale per la fase di import iniziale.
    pub fn lookup_import(&self, word: &str, parent: &str, import_manager: &crate::engine::import::ImportManager) -> Option<&str> {
        let normalized = self.normalize(word);
        if let Some(candidates) = self.keyword_map.get(&normalized) {
            for (canonical, module, language) in candidates {
                let lang_active = language.is_empty() || import_manager.is_member_active(language, "translate");
                if lang_active {
                    let matches_parent = module == parent || (parent == "translate" && module.is_empty());
                    if matches_parent {
                        return Some(canonical.as_str());
                    }
                }
            }
        }
        None
    }

    /// Restituisce il modulo richiesto per una keyword canonica (es. "sin" -> "nmath").
    pub fn required_module(&self, canonical_keyword: &str) -> &str {
        if let Some(module) = self.module_map.get(canonical_keyword) {
            if module == "english"
                || module == "italian"
                || module == "spanish"
                || module == "french"
                || module == "german"
                || module == "portuguese"
                || module == "romanian"
            {
                ""
            } else {
                module.as_str()
            }
        } else {
            ""
        }
    }

    /// Controlla se un identificatore corrisponde a una funzione built-in conosciuta.
    pub fn get_builtin_info(&self, word: &str) -> Option<(&str, &str)> {
        let normalized = self.normalize(word);
        if let Some(candidates) = self.keyword_map.get(&normalized) {
            for (canonical, module, _language) in candidates {
                if module == "nio" || module == "nmath" || module == "nfs" || module == "nnet" {
                    return Some((canonical.as_str(), module.as_str()));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::import::ImportManager;

    #[test]
    fn test_normalization() {
        let engine = TranslationEngine::new();
        assert_eq!(engine.normalize("SÉ"), "se");
        assert_eq!(engine.normalize("funcția"), "functia");
        assert_eq!(engine.normalize("München"), "munchen");
    }

    #[test]
    fn test_mixed_lookups() {
        let engine = TranslationEngine::new();
        let mut import_manager = ImportManager::new();
        import_manager.import_member("english", "translate");
        import_manager.import_member("italian", "translate");
        import_manager.import_member("spanish", "translate");
        import_manager.import_member("romanian", "translate");
        import_manager.import_member("sin", "nmath");

        // English
        assert_eq!(engine.lookup("if", &import_manager), Some("if"));
        // Italian
        assert_eq!(engine.lookup("se", &import_manager), Some("if"));
        // Romanian
        assert_eq!(engine.lookup("daca", &import_manager), Some("if"));
        // Spanish
        assert_eq!(engine.lookup("si", &import_manager), Some("if"));
        
        assert_eq!(engine.required_module("sin"), "nmath");
        assert_eq!(engine.required_module("let"), "");
    }
}
