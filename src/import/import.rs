use std::collections::{HashMap, HashSet};

pub struct ImportManager {
    // Associa un modulo genitore ai suoi sottomoduli/funzioni
    allowed_imports: HashMap<String, HashSet<String>>,
    // Moduli genitori attivi
    active_parents: HashSet<String>,
    // Singoli membri importati singolarmente
    active_members: HashSet<String>,
}

impl ImportManager {
    /// Crea un nuovo ImportManager caricando le associazioni a tempo di compilazione.
    pub fn new() -> Self {
        let json_content = include_str!("import.json");
        let allowed: HashMap<String, Vec<String>> = serde_json::from_str(json_content).unwrap_or_default();
        let allowed_imports = allowed
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();

        Self {
            allowed_imports,
            active_parents: HashSet::new(),
            active_members: HashSet::new(),
        }
    }

    /// Importa un intero modulo genitore (es. `importa nio`).
    pub fn import_all(&mut self, parent: &str) -> bool {
        if let Some(members) = self.allowed_imports.get(parent) {
            self.active_parents.insert(parent.to_string());
            for m in members {
                self.active_members.insert(m.clone());
            }
            true
        } else {
            false
        }
    }

    /// Importa un membro specifico da un modulo genitore.
    /// Il carattere jolly "*" importa tutti i membri.
    pub fn import_member(&mut self, member: &str, parent: &str) -> bool {
        if member == "*" {
            if let Some(members) = self.allowed_imports.get(parent) {
                self.active_parents.insert(parent.to_string());
                for m in members {
                    self.active_members.insert(m.clone());
                }
                return true;
            }
            return false;
        }

        if let Some(members) = self.allowed_imports.get(parent) {
            if members.contains(member) {
                self.active_parents.insert(parent.to_string());
                self.active_members.insert(member.to_string());
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Verifica se un membro specifico è attivo.
    pub fn is_member_active(&self, member: &str, _parent: &str) -> bool {
        self.active_members.contains(member)
    }

    /// Verifica se un modulo genitore è attivo.
    pub fn is_parent_active(&self, parent: &str) -> bool {
        self.active_parents.contains(parent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchical_imports() {
        let mut manager = ImportManager::new();

        // Import con wildcard
        assert!(manager.import_member("*", "translate"));
        assert!(manager.is_member_active("italian", "translate"));
        assert!(manager.is_member_active("english", "translate"));

        // Import di un singolo membro valido
        let mut manager2 = ImportManager::new();
        assert!(manager2.import_member("italian", "translate"));
        assert!(manager2.is_member_active("italian", "translate"));
        assert!(!manager2.is_member_active("english", "translate"));

        // Associazione membro-genitore non valida
        assert!(!manager2.import_member("italian", "english"));
        assert!(!manager2.is_parent_active("english"));

        // Import matematico valido
        assert!(manager2.import_member("sin", "nmath"));
        assert!(manager2.is_member_active("sin", "nmath"));
        assert!(!manager2.is_member_active("cos", "nmath"));
    }
}
