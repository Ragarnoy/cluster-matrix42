//! Build script for the simulator
//!
//! Compiles C and Rust plugins to native shared libraries (.so on Linux, .dylib on macOS, .dll on Windows)

use std::env;
use std::path::PathBuf;
use std::process::Command;

const C_PLUGINS: &[&str] = &["plasma", "quadrant"];
const RUST_PLUGINS: &[&str] = &["bouncing_ball", "quadrant_rust"];

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let plugins_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("plugins");
    let c_plugin_dir = plugins_dir.join("plugin-examples-c");
    let rust_plugin_dir = plugins_dir.join("plugin-examples-rust");

    // Track C source files for rebuild
    println!("cargo:rerun-if-changed={}", c_plugin_dir.display());
    for plugin in C_PLUGINS {
        println!(
            "cargo:rerun-if-changed={}",
            c_plugin_dir.join(format!("{}.c", plugin)).display()
        );
    }
    println!(
        "cargo:rerun-if-changed={}",
        c_plugin_dir.join("common/plugin_api.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        c_plugin_dir.join("common/plugin_helpers.h").display()
    );

    // Track Rust plugin source files for rebuild
    for plugin in RUST_PLUGINS {
        println!(
            "cargo:rerun-if-changed={}",
            rust_plugin_dir.join(plugin).join("src/lib.rs").display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            rust_plugin_dir.join(plugin).join("Cargo.toml").display()
        );
    }

    // Determine shared library extension and prefix based on target OS
    let (lib_prefix, lib_ext) = if cfg!(target_os = "windows") {
        ("", "dll")
    } else if cfg!(target_os = "macos") {
        ("lib", "dylib")
    } else {
        ("lib", "so")
    };

    let mut c_plugins_compiled = Vec::new();
    let mut rust_plugins_compiled = Vec::new();

    // Compile C plugins
    let cc = env::var("CC").unwrap_or_else(|_| "cc".to_string());
    if Command::new(&cc).arg("--version").output().is_ok() {
        for plugin in C_PLUGINS {
            match compile_c_plugin(&c_plugin_dir, &out_dir, plugin, &cc, lib_prefix, lib_ext) {
                Ok(lib_path) => {
                    c_plugins_compiled.push((*plugin, lib_path));
                    println!("cargo:warning=Compiled native C plugin: {}", plugin);
                }
                Err(e) => {
                    println!("cargo:warning=Failed to compile C plugin {}: {}", plugin, e);
                }
            }
        }
    } else {
        println!("cargo:warning=C compiler not found, skipping C plugins");
    }

    // Compile Rust plugins
    for plugin in RUST_PLUGINS {
        match compile_rust_plugin(&rust_plugin_dir, &out_dir, plugin, lib_prefix, lib_ext) {
            Ok(lib_path) => {
                rust_plugins_compiled.push((*plugin, lib_path));
                println!("cargo:warning=Compiled native Rust plugin: {}", plugin);
            }
            Err(e) => {
                println!(
                    "cargo:warning=Failed to compile Rust plugin {}: {}",
                    plugin, e
                );
            }
        }
    }

    generate_plugin_list(&out_dir, &c_plugins_compiled, &rust_plugins_compiled);
}

fn compile_c_plugin(
    src_dir: &PathBuf,
    out_dir: &PathBuf,
    name: &str,
    cc: &str,
    lib_prefix: &str,
    lib_ext: &str,
) -> Result<String, String> {
    let src_file = src_dir.join(format!("{}.c", name));
    let lib_file = out_dir.join(format!("{}{}.{}", lib_prefix, name, lib_ext));
    let include_path = src_dir.join("common");

    if !src_file.exists() {
        return Err(format!("Source file not found: {}", src_file.display()));
    }

    let mut cmd = Command::new(cc);
    cmd.args([
        "-shared",
        "-fPIC",
        "-O2",
        "-I",
        include_path.to_str().unwrap(),
        src_file.to_str().unwrap(),
        "-o",
        lib_file.to_str().unwrap(),
    ]);

    if cfg!(target_os = "macos") {
        cmd.arg("-undefined").arg("dynamic_lookup");
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run {}: {}", cc, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Compilation failed: {}", stderr));
    }

    Ok(lib_file.to_string_lossy().to_string())
}

fn compile_rust_plugin(
    rust_plugin_dir: &PathBuf,
    out_dir: &PathBuf,
    name: &str,
    lib_prefix: &str,
    lib_ext: &str,
) -> Result<String, String> {
    let plugin_dir = rust_plugin_dir.join(name);

    if !plugin_dir.exists() {
        return Err(format!(
            "Plugin directory not found: {}",
            plugin_dir.display()
        ));
    }

    // Build the plugin as a cdylib with the simulator feature
    let output = Command::new("cargo")
        .args([
            "build",
            "--lib",
            "--release",
            "--features",
            "simulator",
            "--manifest-path",
            plugin_dir.join("Cargo.toml").to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run cargo: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Cargo build failed: {}", stderr));
    }

    // Find the built library
    // Rust uses underscores in library names, not hyphens
    let lib_name = name.replace('-', "_");
    let rust_lib = rust_plugin_dir
        .join("target/release")
        .join(format!("{}{}.{}", lib_prefix, lib_name, lib_ext));

    if !rust_lib.exists() {
        return Err(format!(
            "Built library not found at: {}",
            rust_lib.display()
        ));
    }

    // Copy to out_dir
    let dest = out_dir.join(format!("{}{}.{}", lib_prefix, lib_name, lib_ext));
    std::fs::copy(&rust_lib, &dest).map_err(|e| format!("Failed to copy library: {}", e))?;

    Ok(dest.to_string_lossy().to_string())
}

fn generate_plugin_list(
    out_dir: &PathBuf,
    c_plugins: &[(&str, String)],
    rust_plugins: &[(&str, String)],
) {
    let mut code = String::new();

    // C plugins (use name-prefixed symbols: plasma_init, plasma_update, etc.)
    code.push_str("/// List of compiled native C plugins (name, path, uses_prefixed_symbols)\n");
    code.push_str("pub const NATIVE_C_PLUGINS: &[(&str, &str)] = &[\n");
    for (name, path) in c_plugins {
        code.push_str(&format!("    (\"{}\", \"{}\"),\n", name, path));
    }
    code.push_str("];\n\n");

    // Rust plugins (use __plugin_* symbols)
    code.push_str("/// List of compiled native Rust plugins (name, path)\n");
    code.push_str("pub const NATIVE_RUST_PLUGINS: &[(&str, &str)] = &[\n");
    for (name, path) in rust_plugins {
        code.push_str(&format!("    (\"{}\", \"{}\"),\n", name, path));
    }
    code.push_str("];\n");

    std::fs::write(out_dir.join("native_plugins.rs"), code).unwrap();
}
