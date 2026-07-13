/// Lista delle funzioni built-in di NodeStract — fonte unica di verità.
///
/// Ogni funzione è elencata una sola volta qui. Sia `is_function_defined` nell'interprete
/// che il rilevamento in `expressions.rs` leggono da questa costante, eliminando la
/// duplicazione manuale della stessa lista in più punti del codice.
pub const BUILTIN_NAMES: &[&str] = &[
    // Modulo nio — Input/Output
    "print", "input",
    // Modulo nfs — File System
    "read", "write", "delete",
    // Modulo nmath — Matematica
    "sin", "cos", "sqrt", "random", "round", "min", "max", "abs", "log", "pow",
    // Modulo nnet — Rete
    "fetch", "send",
    // Funzioni universali (nessun modulo richiesto — sempre disponibili se la lingua è importata)
    "len", "sleep", "exit",
];

/// Verifica se un nome corrisponde a una funzione built-in di NodeStract.
pub fn is_builtin(name: &str) -> bool {
    BUILTIN_NAMES.contains(&name)
}
