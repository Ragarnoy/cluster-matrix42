//! Embedded entry point for bouncing_ball plugin
//!
//! This is a thin wrapper that provides the no_std entry point for embedded targets.
//! The actual plugin logic is in lib.rs.
//!
//! This file is only compiled for embedded targets (not simulator).

#![cfg_attr(not(feature = "simulator"), no_std)]
#![cfg_attr(not(feature = "simulator"), no_main)]

// Re-export the plugin from lib.rs - this brings in the plugin_main! generated symbols
pub use bouncing_ball::*;

#[cfg(not(feature = "simulator"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(feature = "simulator")]
fn main() {
    // This binary target is not used for simulator builds.
    // The cdylib target (lib.rs) is used instead.
    eprintln!("This binary is for embedded targets only.");
    eprintln!("Use the shared library (.so/.dylib) for simulator.");
}
