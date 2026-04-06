#![no_std]
#![doc = "cluster-net: REST API client library for cluster-matrix"]
#![doc = ""]
#![doc = "A no_std library for making HTTP requests to a cluster server."]
#![doc = "Provides read-only access to cluster data via REST API."]

#[cfg(feature = "std")]
extern crate std;

pub mod client;
pub mod endpoints;
pub mod error;

#[cfg(feature = "tls")]
pub mod tls;

// Re-export commonly used types
pub use client::Client;
pub use error::{Error, Result};

#[cfg(feature = "tls")]
pub use tls::{create_tls_config, create_tls_config_with_psk};

/// Default buffer size for HTTP responses (8KB)
pub const DEFAULT_BUFFER_SIZE: usize = 8192;

/// Maximum URL length
pub const MAX_URL_LENGTH: usize = 256;

/// Maximum number of headers in a request
pub const MAX_HEADERS: usize = 8;
