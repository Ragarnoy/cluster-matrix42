use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_path = PathBuf::from(&crate_dir)
        .parent()
        .unwrap()
        .join("plugin-examples-c")
        .join("common");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&out_path).unwrap();

    let config = cbindgen::Config::from_file("cbindgen.toml").unwrap();

    let builder = cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(config);

    match builder.generate() {
        Ok(bindings) => {
            bindings.write_to_file(out_path.join("plugin_api.h"));
        }
        Err(e) => {
            panic!("Failed to generate C bindings: {:?}", e);
        }
    }

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
}
