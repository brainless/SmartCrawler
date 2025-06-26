use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Generate TypeScript types during build
    let out_dir = env::var("OUT_DIR").unwrap();
    let ts_dir = Path::new(&out_dir).join("ts");

    // Create ts directory if it doesn't exist
    fs::create_dir_all(&ts_dir).unwrap();

    // Export all the types to TypeScript files
    println!("cargo:rerun-if-changed=src/entities.rs");

    // The actual export is handled by the #[ts(export)] attributes
    // ts-rs will generate the .ts files when the library is built
}
