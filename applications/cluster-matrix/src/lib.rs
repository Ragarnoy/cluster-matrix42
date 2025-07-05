#![no_std]

#[cfg(feature = "std")]
extern crate std;

// Re-export the separated crates for backwards compatibility
pub use cluster_core as cluster;
pub use graphics_common as graphics;

// Legacy re-exports to maintain compatibility
pub use cluster_core::{constants, shared, visualization};
pub use graphics_common::{animations, utilities};
