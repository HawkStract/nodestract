const COLOR_RESET: &str = "\x1b[0m";
const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_CYAN: &str = "\x1b[36m";

pub fn show_banner() {
    println!("{}
    _   _           _        ____  _                  _   
   | \\ | | ___   __| | ___  / ___|| |_ _ __ __ _  ___| |_ 
   |  \\| |/ _  \\/ _` |/ _ \\ \\___ \\| __| '__/ _` |/ __| __|
   | |\\  | (_) \\ (_| |  __/  ___) | |_| | | (_| | (__| |_ 
   |_| \\_|\\___/ \\__,_|\\___| |____/ \\__|_|  \\__,_|\\___|\\__|
   
   {}", COLOR_CYAN, COLOR_RESET);
}

pub fn show_usage() {
    println!("Usage:");
    println!("  cargo run -- build <file.ns>           Compile and run a NodeStract file");
    println!("  cargo run -- version                   Show version information");
    println!("  cargo run --example lessons            Launch the interactive lessons mode");
    println!("  cargo test                             Run the unit and integration test suite");
    println!("\nLanguages supported concurrently: en, it, es, fr, de, pt, ro");
}

pub fn show_welcome() {
    show_banner();
    show_usage();
}

pub fn show_version() {
    println!("Node Stract Compiler (NSC) v{} - HawkStract Ecosystem", env!("CARGO_PKG_VERSION"));
}

pub fn show_error(msg: &str) {
    println!("{}Error: {}{}", COLOR_RED, msg, COLOR_RESET);
}

pub fn show_success(msg: &str) {
    println!("{}{} [Success]{}", COLOR_GREEN, msg, COLOR_RESET);
}
