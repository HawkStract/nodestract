use std::fs;
use std::path::Path;

fn main() {

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_tests.rs");
    let mut test_code = String::new();

    let categories = &["languages", "typing", "data", "net", "function", "conditional", "logical", "math", "file"];
    for &category in categories {
        let dir_path = format!("examples/test/{}", category);
        if let Ok(entries) = fs::read_dir(&dir_path) {
            let mut paths = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "ns") {
                        paths.push(path);
                    }
                }
            }
            paths.sort();

            for path in paths {
                let file_name = path.file_stem().unwrap().to_str().unwrap();
                let clean_name = format!("{}_{}", category, file_name)
                    .replace(".", "_")
                    .replace("-", "_")
                    .replace(" ", "_");
                let path_str = path.to_str().unwrap().replace("\\", "/");
                
                test_code.push_str(&format!(
                    "#[test]\nfn test_{}() {{\n    run_one_test(\"{}\");\n}}\n\n",
                    clean_name, path_str
                ));
            }
        }
    }

    fs::write(&dest_path, test_code).unwrap();
    println!("cargo:rerun-if-changed=examples/test");
}