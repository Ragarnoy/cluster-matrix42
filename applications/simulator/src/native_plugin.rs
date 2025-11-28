//! Native plugin loader
//!
//! Loads C and Rust plugins compiled as shared libraries (.so/.dylib/.dll)
//! and wraps them in the Plugin trait.
//!
//! C plugins use name-prefixed symbols: `{name}_init`, `{name}_update`, `{name}_cleanup`
//! Rust plugins use generic symbols: `__plugin_init`, `__plugin_update`, `__plugin_cleanup`

use crate::plugin_host::Plugin;
use libloading::{Library, Symbol};
use plugin_api::{Inputs, PluginAPI};
use std::path::Path;

// Include the list of compiled native plugins from build.rs
include!(concat!(env!("OUT_DIR"), "/native_plugins.rs"));

/// Symbol naming convention for plugins
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolConvention {
    /// C-style: `{name}_init`, `{name}_update`, `{name}_cleanup`
    NamePrefixed,
    /// Rust-style: `__plugin_init`, `__plugin_update`, `__plugin_cleanup`
    Generic,
}

/// A plugin loaded from a shared library
pub struct NativePlugin {
    _lib: Library,
    name: String,
    init_fn: Symbol<'static, unsafe extern "C" fn(*const PluginAPI) -> i32>,
    update_fn: Symbol<'static, unsafe extern "C" fn(*const PluginAPI, u32)>,
    cleanup_fn: Symbol<'static, unsafe extern "C" fn()>,
}

impl NativePlugin {
    /// Load a plugin from a shared library with the specified symbol convention
    pub fn load(path: &Path, name: &str, convention: SymbolConvention) -> Result<Self, String> {
        unsafe {
            let lib = Library::new(path).map_err(|e| format!("Failed to load library: {}", e))?;

            // Build symbol names based on convention
            let (init_name, update_name, cleanup_name) = match convention {
                SymbolConvention::NamePrefixed => (
                    format!("{}_init\0", name),
                    format!("{}_update\0", name),
                    format!("{}_cleanup\0", name),
                ),
                SymbolConvention::Generic => (
                    "__plugin_init\0".to_string(),
                    "__plugin_update\0".to_string(),
                    "__plugin_cleanup\0".to_string(),
                ),
            };

            // Load function symbols
            // We need to transmute the lifetime to 'static because we're storing them
            // This is safe because we keep _lib alive for the lifetime of NativePlugin
            let init_fn: Symbol<unsafe extern "C" fn(*const PluginAPI) -> i32> = lib
                .get(init_name.as_bytes())
                .map_err(|e| format!("Failed to find init symbol: {}", e))?;
            let init_fn: Symbol<'static, unsafe extern "C" fn(*const PluginAPI) -> i32> =
                std::mem::transmute(init_fn);

            let update_fn: Symbol<unsafe extern "C" fn(*const PluginAPI, u32)> = lib
                .get(update_name.as_bytes())
                .map_err(|e| format!("Failed to find update symbol: {}", e))?;
            let update_fn: Symbol<'static, unsafe extern "C" fn(*const PluginAPI, u32)> =
                std::mem::transmute(update_fn);

            let cleanup_fn: Symbol<unsafe extern "C" fn()> = lib
                .get(cleanup_name.as_bytes())
                .map_err(|e| format!("Failed to find cleanup symbol: {}", e))?;
            let cleanup_fn: Symbol<'static, unsafe extern "C" fn()> =
                std::mem::transmute(cleanup_fn);

            Ok(Self {
                _lib: lib,
                name: name.to_string(),
                init_fn,
                update_fn,
                cleanup_fn,
            })
        }
    }

    /// Load a C plugin by name
    pub fn load_c_plugin(name: &str) -> Result<Self, String> {
        for (plugin_name, path) in NATIVE_C_PLUGINS {
            if *plugin_name == name {
                return Self::load(Path::new(path), name, SymbolConvention::NamePrefixed);
            }
        }
        Err(format!("C plugin '{}' not found", name))
    }

    /// Load a Rust plugin by name
    pub fn load_rust_plugin(name: &str) -> Result<Self, String> {
        for (plugin_name, path) in NATIVE_RUST_PLUGINS {
            if *plugin_name == name {
                return Self::load(Path::new(path), name, SymbolConvention::Generic);
            }
        }
        Err(format!("Rust plugin '{}' not found", name))
    }

    /// Get list of available C plugins
    pub fn available_c_plugins() -> &'static [(&'static str, &'static str)] {
        NATIVE_C_PLUGINS
    }

    /// Get list of available Rust plugins
    pub fn available_rust_plugins() -> &'static [(&'static str, &'static str)] {
        NATIVE_RUST_PLUGINS
    }

    /// Get all available plugins (C and Rust)
    pub fn all_available_plugins() -> Vec<(&'static str, bool)> {
        let mut plugins = Vec::new();
        for (name, _) in NATIVE_C_PLUGINS {
            plugins.push((*name, true)); // true = is C
        }
        for (name, _) in NATIVE_RUST_PLUGINS {
            plugins.push((*name, false)); // false = is Rust
        }
        plugins
    }
}

impl Plugin for NativePlugin {
    fn new() -> Self
    where
        Self: Sized,
    {
        panic!("NativePlugin::new() is not supported, use load_c_plugin() or load_rust_plugin()")
    }

    fn init(&mut self, api: &mut PluginAPI) -> i32 {
        unsafe { (self.init_fn)(api as *const PluginAPI) }
    }

    fn update(&mut self, api: &mut PluginAPI, inputs: Inputs) {
        unsafe { (self.update_fn)(api as *const PluginAPI, inputs.raw()) }
    }

    fn cleanup(&mut self) {
        unsafe { (self.cleanup_fn)() }
    }

    fn name(&self) -> &'static str {
        Box::leak(self.name.clone().into_boxed_str())
    }
}
