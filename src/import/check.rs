use crate::engine::import::ImportManager;
use crate::engine::translate::TranslationEngine;

/// Valida gli import all'inizio del file, li registra e restituisce il sorgente ripulito dagli import.
pub fn validate_imports(
    source: &str,
    translation_engine: &TranslationEngine,
) -> Result<(String, ImportManager), String> {
    let mut import_manager = ImportManager::new();
    let mut stripped_lines = Vec::new();
    let mut imports_ended = false;
    let mut inside_multiline_comment = false;
    let mut has_imported_any_language = false;

    for (line_num, line) in source.lines().enumerate() {
        let mut clean_line = line.to_string();
        if let Some(pos) = clean_line.find("//") {
            clean_line = clean_line[..pos].to_string();
        }
        let trimmed = clean_line.trim();

        // Gestione dei commenti multilinea
        if inside_multiline_comment {
            if trimmed.contains("*/") {
                inside_multiline_comment = false;
            }
            stripped_lines.push(line.to_string());
            continue;
        }

        if trimmed.starts_with("/*") {
            inside_multiline_comment = true;
            if trimmed.contains("*/") {
                inside_multiline_comment = false;
            }
            stripped_lines.push(line.to_string());
            continue;
        }

        // Ignora linee vuote o commenti completi
        if trimmed.is_empty() {
            stripped_lines.push(line.to_string());
            continue;
        }

        // Split line into words to check for import
        let words: Vec<&str> = trimmed
            .split_whitespace()
            .map(|w| w.trim_matches(|c| c == ';' || c == ','))
            .filter(|w| !w.is_empty())
            .collect();

        if words.is_empty() {
            stripped_lines.push(line.to_string());
            continue;
        }

        // Controlla se la prima parola corrisponde a "import"
        let temp_manager = ImportManager::new();
        let is_import_stmt = translation_engine
            .lookup(words[0], &temp_manager)
            .map_or(false, |kw| kw == "import");

        if is_import_stmt {
            if imports_ended {
                return Err(format!(
                    "Syntax Error (Line {}): Import statement found after non-import code",
                    line_num + 1
                ));
            }

            let len = words.len();
            if len < 4 {
                return Err(format!(
                    "Syntax Error (Line {}): Invalid import syntax. Expected: import <member(s)> from <parent>",
                    line_num + 1
                ));
            }

            let parent = words[len - 1];
            let from_keyword = words[len - 2];

            // Controlla il keyword "from"
            let is_from = translation_engine
                .lookup(from_keyword, &temp_manager)
                .map_or(false, |kw| kw == "from");

            if !is_from {
                return Err(format!(
                    "Syntax Error (Line {}): Expected 'from' keyword, found '{}'",
                    line_num + 1,
                    from_keyword
                ));
            }

            let members = &words[1..len - 2];
            for &member in members {
                // Traduce il membro nella forma inglese canonica (es. "italiano" -> "italian")
                let canonical_member = translation_engine
                    .lookup_import(member, parent, &import_manager)
                    .unwrap_or(member);

                // Registra l'import
                if !import_manager.import_member(canonical_member, parent) {
                    return Err(format!(
                        "Import Error (Line {}): Cannot import '{}' from '{}'",
                        line_num + 1,
                        member,
                        parent
                    ));
                }

                if parent == "translate" {
                    has_imported_any_language = true;
                }
            }

            // Sostituisce la riga con una vuota per mantenere corretti i numeri di riga negli errori
            stripped_lines.push("".to_string());
        } else {
            // Trovata la prima riga di codice reale, chiude la fase di import
            imports_ended = true;
            stripped_lines.push(line.to_string());
        }
    }

    if !has_imported_any_language {
        return Err(
            "Import Error: No language imported. You must import at least one language (e.g., 'english', 'italian') from 'translate' at the beginning of the file."
                .to_string(),
        );
    }

    Ok((stripped_lines.join("\n"), import_manager))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_count_preservation() {
        let source = "importa italiano da translate\n// Un commento\n\nscrivi(\"ciao\")";
        let translation_engine = TranslationEngine::new();
        let (stripped, _) = validate_imports(source, &translation_engine).unwrap();
        
        let original_lines: Vec<&str> = source.lines().collect();
        let stripped_lines: Vec<&str> = stripped.lines().collect();
        
        assert_eq!(original_lines.len(), stripped_lines.len());
        // Verifica che la riga di import sia vuota e le successive intatte
        assert_eq!(stripped_lines[0], "");
        assert_eq!(stripped_lines[1], "// Un commento");
        assert_eq!(stripped_lines[2], "");
        assert_eq!(stripped_lines[3], "scrivi(\"ciao\")");
    }
}
