use std::collections::HashMap;

pub struct TranslationEngine {
    // Maps lowercase, accent-normalized keywords to their canonical English forms
    keyword_map: HashMap<String, String>,
}

impl TranslationEngine {
    /// Create a new translation engine, loading English by default, plus the target language.
    pub fn new(lang: &str) -> Self {
        let mut engine = Self {
            keyword_map: HashMap::new(),
        };
        // Always load English as fallback/standard
        engine.load_language("en");
        if lang != "en" {
            engine.load_language(lang);
        }
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

        if let Ok(map) = serde_json::from_str::<HashMap<String, Vec<String>>>(json_content) {
            for (canonical_kw, translations) in map {
                for translation in translations {
                    let normalized = self.normalize(&translation);
                    self.keyword_map.insert(normalized, canonical_kw.clone());
                }
            }
        }
    }

    /// Normalizes a string by converting it to lowercase and replacing accented/diacritic characters
    /// with their closest base ASCII equivalent.
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

    /// Resolves a localized keyword to its canonical English form (e.g. "function" -> "func", "se" -> "if").
    pub fn lookup(&self, word: &str) -> Option<&str> {
        let normalized = self.normalize(word);
        self.keyword_map.get(&normalized).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let engine = TranslationEngine::new("en");
        assert_eq!(engine.normalize("SÉ"), "se");
        assert_eq!(engine.normalize("funcția"), "functia");
        assert_eq!(engine.normalize("München"), "munchen");
    }

    #[test]
    fn test_english_lookup() {
        let engine = TranslationEngine::new("en");
        assert_eq!(engine.lookup("if"), Some("if"));
        assert_eq!(engine.lookup("function"), Some("function"));
        assert_eq!(engine.lookup("let"), Some("let"));
        assert_eq!(engine.lookup("const"), Some("const"));
        assert_eq!(engine.lookup("fetch"), Some("fetch"));
        assert_eq!(engine.lookup("sin"), Some("sin"));
        assert_eq!(engine.lookup("not_a_keyword"), None);
    }
}

