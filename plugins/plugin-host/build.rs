use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const PLUGIN_NAMES: &[&str] = &["plasma", "quadrant"];

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let plugin_src_dir = manifest_dir.parent().unwrap().join("plugin-examples-c");
    let header_file = plugin_src_dir.join("common").join("plugin_api.h");

    // Track C source files and headers for rebuild
    println!("cargo:rerun-if-changed={}", header_file.display());
    println!(
        "cargo:rerun-if-changed={}",
        plugin_src_dir.join("common/plugin_helpers.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        plugin_src_dir.join("common/plugin.ld").display()
    );
    for plugin in PLUGIN_NAMES {
        println!(
            "cargo:rerun-if-changed={}",
            plugin_src_dir.join(format!("{}.c", plugin)).display()
        );
    }

    // Check if GCC is available
    if Command::new("arm-none-eabi-gcc")
        .arg("--version")
        .output()
        .is_err()
    {
        println!("cargo:warning=arm-none-eabi-gcc not found, skipping plugin compilation");
        generate_empty_plugin_list(&out_dir, &target);
        return;
    }

    if !header_file.exists() {
        println!("cargo:warning=plugin_api.h not found, skipping plugin compilation");
        generate_empty_plugin_list(&out_dir, &target);
        return;
    }

    if target.contains("thumbv8m") {
        let mut successful_plugins = Vec::new();

        for plugin in PLUGIN_NAMES {
            match compile_plugin(&plugin_src_dir, &out_dir, plugin) {
                Ok(()) => {
                    successful_plugins.push(*plugin);
                    println!("cargo:warning=Successfully compiled plugin: {}", plugin);
                }
                Err(e) => {
                    println!("cargo:warning=Failed to compile plugin {}: {}", plugin, e);
                }
            }
        }

        if successful_plugins.is_empty() {
            generate_empty_plugin_list(&out_dir, &target);
        } else {
            generate_plugin_includes(&out_dir, &successful_plugins);
        }
    } else {
        generate_empty_plugin_list(&out_dir, &target);
    }
}

fn compile_plugin(src_dir: &Path, out_dir: &Path, name: &str) -> Result<(), String> {
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

    // Only continue if object file was created
    if !obj_file.exists() {
        return Err("Object file was not created".to_string());
    }

    // Link - create a minimal linker script if needed
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

fn generate_empty_plugin_list(out_dir: &Path, _target: &str) {
    let code = r#"
        #[cfg(target_arch = "arm")]
        pub mod plugins {}

        pub fn get_plugin_list() -> &'static [(&'static str, &'static [u8])] {
            &[]
        }
    "#;
    std::fs::write(out_dir.join("plugin_includes.rs"), code).unwrap();
}

fn generate_plugin_includes(out_dir: &Path, plugins: &[&str]) {
    let mut code = String::from("pub mod plugins {\n");
    for plugin in plugins {
        code.push_str(&format!(
            "    pub const {}_BYTES: &[u8] = include_bytes!(\"{}/{}.bin\");\n",
            plugin.to_uppercase(),
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
            plugin.to_uppercase()
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
