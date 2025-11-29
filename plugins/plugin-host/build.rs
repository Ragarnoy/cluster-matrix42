use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let c_plugin_dir = manifest_dir.parent().unwrap().join("plugin-examples-c");
    let rust_plugin_dir = manifest_dir.parent().unwrap().join("plugin-examples-rust");

    // Auto-discover C plugins (any .c file in plugin-examples-c, excluding common/)
    let c_plugins = discover_c_plugins(&c_plugin_dir);
    // Auto-discover Rust plugins (any subdirectory with Cargo.toml in plugin-examples-rust)
    let rust_plugins = discover_rust_plugins(&rust_plugin_dir);

    // Track directories for rebuild on new plugin addition
    println!("cargo:rerun-if-changed={}", c_plugin_dir.display());
    println!("cargo:rerun-if-changed={}", rust_plugin_dir.display());

    // Track C source files and headers for rebuild
    let header_file = c_plugin_dir.join("common").join("plugin_api.h");
    println!("cargo:rerun-if-changed={}", header_file.display());
    println!(
        "cargo:rerun-if-changed={}",
        c_plugin_dir.join("common/plugin_helpers.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        c_plugin_dir.join("common/plugin.ld").display()
    );
    for plugin in &c_plugins {
        println!(
            "cargo:rerun-if-changed={}",
            c_plugin_dir.join(format!("{}.c", plugin)).display()
        );
    }

    // Track Rust plugin source files for rebuild
    for plugin in &rust_plugins {
        println!(
            "cargo:rerun-if-changed={}",
            rust_plugin_dir.join(plugin).join("src/lib.rs").display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            rust_plugin_dir.join(plugin).join("src/main.rs").display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            rust_plugin_dir.join(plugin).join("Cargo.toml").display()
        );
    }

    if !target.contains("thumbv8m") {
        generate_empty_plugin_list(&out_dir);
        return;
    }

    let mut successful_plugins = Vec::new();

    // Compile C plugins
    if Command::new("arm-none-eabi-gcc")
        .arg("--version")
        .output()
        .is_ok()
        && header_file.exists()
    {
        for plugin in &c_plugins {
            match compile_c_plugin(&c_plugin_dir, &out_dir, plugin) {
                Ok(()) => {
                    successful_plugins.push(plugin.clone());
                    println!("cargo:warning=Successfully compiled C plugin: {}", plugin);
                }
                Err(e) => {
                    println!("cargo:warning=Failed to compile C plugin {}: {}", plugin, e);
                }
            }
        }
    } else {
        println!("cargo:warning=arm-none-eabi-gcc not found or header missing, skipping C plugins");
    }

    // Compile Rust plugins
    for plugin in &rust_plugins {
        match compile_rust_plugin(&rust_plugin_dir, &out_dir, plugin) {
            Ok(()) => {
                successful_plugins.push(plugin.clone());
                println!(
                    "cargo:warning=Successfully compiled Rust plugin: {}",
                    plugin
                );
            }
            Err(e) => {
                println!(
                    "cargo:warning=Failed to compile Rust plugin {}: {}",
                    plugin, e
                );
            }
        }
    }

    if successful_plugins.is_empty() {
        generate_empty_plugin_list(&out_dir);
    } else {
        generate_plugin_includes(&out_dir, &successful_plugins);
    }
}

/// Discover C plugins by scanning for .c files in the plugin directory
fn discover_c_plugins(c_plugin_dir: &Path) -> Vec<String> {
    let mut plugins = Vec::new();

    if let Ok(entries) = std::fs::read_dir(c_plugin_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension()
                && ext == "c"
                && let Some(stem) = path.file_stem()
            {
                plugins.push(stem.to_string_lossy().to_string());
            }
        }
    }

    plugins.sort();
    plugins
}

/// Discover Rust plugins by scanning for subdirectories with Cargo.toml
fn discover_rust_plugins(rust_plugin_dir: &Path) -> Vec<String> {
    let mut plugins = Vec::new();

    if let Ok(entries) = std::fs::read_dir(rust_plugin_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && path.join("Cargo.toml").exists()
                && let Some(name) = path.file_name()
            {
                plugins.push(name.to_string_lossy().to_string());
            }
        }
    }

    plugins.sort();
    plugins
}

fn compile_c_plugin(src_dir: &Path, out_dir: &Path, name: &str) -> Result<(), String> {
    let src_file = src_dir.join(format!("{}.c", name));

    if !src_file.exists() {
        return Err("Source file does not exist".to_string());
    }

    let obj_file = out_dir.join(format!("{}.o", name));
    let elf_file = out_dir.join(format!("{}.elf", name));
    let bin_file = out_dir.join(format!("{}.bin", name));

    let include_path = src_dir.join("common");

    let output = Command::new("arm-none-eabi-gcc")
        .args([
            "-mcpu=cortex-m33",
            "-mthumb",
            "-fPIC",
            "-ffreestanding",
            "-nostdlib",
            "-O2",
            "-mfloat-abi=hard",
            "-I",
            include_path.to_str().unwrap(),
            "-c",
            src_file.to_str().unwrap(),
            "-o",
            obj_file.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run gcc: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("cargo:warning=GCC compilation failed for {}:", name);
        for line in stderr.lines() {
            println!("cargo:warning=  {}", line);
        }
        return Err(format!(
            "Compilation failed with exit code: {:?}",
            output.status.code()
        ));
    }

    if !obj_file.exists() {
        return Err("Object file was not created".to_string());
    }

    // Link
    let ld_script = src_dir.join("common/plugin.ld");
    if !ld_script.exists() {
        println!("cargo:warning=Creating default linker script");
        std::fs::create_dir_all(src_dir.join("common")).ok();
        std::fs::write(&ld_script, DEFAULT_LINKER_SCRIPT).map_err(|e| e.to_string())?;
    }

    let output = Command::new("arm-none-eabi-ld")
        .args([
            "-T",
            ld_script.to_str().unwrap(),
            obj_file.to_str().unwrap(),
            "-o",
            elf_file.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run ld: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("cargo:warning=Linker error: {}", stderr);
        return Err("Linking failed".to_string());
    }

    // Convert to binary
    Command::new("arm-none-eabi-objcopy")
        .args([
            "-O",
            "binary",
            elf_file.to_str().unwrap(),
            bin_file.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| format!("objcopy failed: {}", e))?;

    if let Ok(metadata) = std::fs::metadata(&bin_file) {
        println!(
            "cargo:warning=Plugin {} size: {} bytes",
            name,
            metadata.len()
        );
    }

    Ok(())
}

fn compile_rust_plugin(rust_plugin_dir: &Path, out_dir: &Path, name: &str) -> Result<(), String> {
    let plugin_dir = rust_plugin_dir.join(name);

    if !plugin_dir.exists() {
        return Err(format!(
            "Plugin directory not found: {}",
            plugin_dir.display()
        ));
    }

    // Build the plugin with cargo for the embedded target
    // Uses pre-installed target (rustup target add thumbv8m.main-none-eabihf)
    let output = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target",
            "thumbv8m.main-none-eabihf",
            "--manifest-path",
            plugin_dir.join("Cargo.toml").to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run cargo: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("cargo:warning=Cargo build failed for {}:", name);
        for line in stderr.lines().take(20) {
            println!("cargo:warning=  {}", line);
        }
        return Err("Cargo build failed".to_string());
    }

    // Find the built ELF file
    let elf_file = rust_plugin_dir
        .join("target/thumbv8m.main-none-eabihf/release")
        .join(name);

    if !elf_file.exists() {
        return Err(format!("Built ELF not found at: {}", elf_file.display()));
    }

    // Convert ELF to binary
    let bin_file = out_dir.join(format!("{}.bin", name));

    let output = Command::new("arm-none-eabi-objcopy")
        .args([
            "-O",
            "binary",
            elf_file.to_str().unwrap(),
            bin_file.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("objcopy failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("objcopy failed: {}", stderr));
    }

    if let Ok(metadata) = std::fs::metadata(&bin_file) {
        println!(
            "cargo:warning=Plugin {} size: {} bytes",
            name,
            metadata.len()
        );
    }

    Ok(())
}

fn generate_empty_plugin_list(out_dir: &Path) {
    let code = r#"
        #[cfg(target_arch = "arm")]
        pub mod plugins {}

        pub fn get_plugin_list() -> &'static [(&'static str, &'static [u8])] {
            &[]
        }
    "#;
    std::fs::write(out_dir.join("plugin_includes.rs"), code).unwrap();
}

fn generate_plugin_includes(out_dir: &Path, plugins: &[String]) {
    let mut code = String::from("pub mod plugins {\n");
    for plugin in plugins {
        code.push_str(&format!(
            "    pub const {}_BYTES: &[u8] = include_bytes!(\"{}/{}.bin\");\n",
            plugin.to_uppercase().replace('-', "_"),
            out_dir.display(),
            plugin
        ));
    }
    code.push_str("}\n\n");
    code.push_str(
        "pub fn get_plugin_list() -> &'static [(&'static str, &'static [u8])] {\n    &[\n",
    );
    for plugin in plugins {
        code.push_str(&format!(
            "        (\"{}\", plugins::{}_BYTES),\n",
            plugin,
            plugin.to_uppercase().replace('-', "_")
        ));
    }
    code.push_str("    ]\n}\n");
    std::fs::write(out_dir.join("plugin_includes.rs"), code).unwrap();
}

const DEFAULT_LINKER_SCRIPT: &str = r#"
MEMORY {
    PLUGIN : ORIGIN = 0x00000000, LENGTH = 64K
}

SECTIONS {
    .plugin_header : {
        KEEP(*(.plugin_header))
    } > PLUGIN

    .text : {
        *(.text*)
        *(.rodata*)
    } > PLUGIN

    .data : {
        *(.data*)
    } > PLUGIN
}
"#;
