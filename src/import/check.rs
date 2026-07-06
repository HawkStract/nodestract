use crate::engine::import::ImportManager;
use crate::engine::translate::TranslationEngine;

/// Validates the import block at the beginning of the file, registers active imports,
/// and returns the stripped source code along with the populated ImportManager.
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
        let trimmed = line.trim();

        // Handle multi-line comment state
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

        // Skip empty lines or single-line comments
        if trimmed.is_empty() || trimmed.starts_with("//") {
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

        // Check if the first word maps to the canonical "import" keyword
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

            // Verify "from" keyword
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
                // Translate member to its canonical English form (e.g. "italiano" -> "italian")
                let canonical_member = translation_engine
                    .lookup_import(member, parent, &import_manager)
                    .unwrap_or(member);

                // Register import in ImportManager
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

            // Preserve line count by replacing import line with an empty line
            stripped_lines.push("".to_string());
        } else {
            // First non-import line found. All future imports are illegal.
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
        // Verify that the import line is now empty
        assert_eq!(stripped_lines[0], "");
        // Verify that the subsequent lines are intact and on the same line index
        assert_eq!(stripped_lines[1], "// Un commento");
        assert_eq!(stripped_lines[2], "");
        assert_eq!(stripped_lines[3], "scrivi(\"ciao\")");
    }
}
