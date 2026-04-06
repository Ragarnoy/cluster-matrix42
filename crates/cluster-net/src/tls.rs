//! TLS configuration helpers
//!
//! This module provides utilities for configuring TLS connections.

use reqwless::client::{TlsConfig, TlsVerify};

/// Re-export embedded-tls types for convenience
pub use embedded_tls::{Aes128GcmSha256, Aes256GcmSha384, TlsCipherSuite, TlsVerifier};

/// Maximum read/write buffer size for TLS (16KB)
pub const TLS_BUFFER_SIZE: usize = 16384;

/// Recommended seed for TLS (can be any value for deterministic behavior)
pub const DEFAULT_TLS_SEED: u64 = 0;

/// Helper to create a TLS configuration without verification (for testing)
///
/// Creates a TlsConfig with the provided buffers.
/// The buffers must be at least 16KB each for reliable operation.
///
/// **Warning:** This disables certificate verification, making the connection
/// vulnerable to man-in-the-middle attacks. Only use for testing!
///
/// # Arguments
/// * `read_buffer` - Buffer for reading TLS records (minimum 16KB recommended)
/// * `write_buffer` - Buffer for writing TLS records (minimum 16KB recommended)
///
/// # Example
/// ```no_run
/// # #[cfg(feature = "tls")] {
/// use cluster_net::tls::{create_tls_config, TLS_BUFFER_SIZE};
///
/// # fn example() {
/// let mut rx_buf = [0u8; TLS_BUFFER_SIZE];
/// let mut tx_buf = [0u8; TLS_BUFFER_SIZE];
///
/// let tls = create_tls_config(&mut rx_buf, &mut tx_buf);
/// # }
/// # }
/// ```
pub fn create_tls_config<'a>(
    read_buffer: &'a mut [u8],
    write_buffer: &'a mut [u8],
) -> TlsConfig<'a> {
    TlsConfig::new(DEFAULT_TLS_SEED, read_buffer, write_buffer, TlsVerify::None)
}

/// Helper to create a TLS configuration with PSK (Pre-Shared Key) verification
///
/// Creates a TlsConfig using pre-shared keys for authentication.
/// This is more suitable for embedded systems than certificate-based TLS.
///
/// # Arguments
/// * `read_buffer` - Buffer for reading TLS records (minimum 16KB recommended)
/// * `write_buffer` - Buffer for writing TLS records (minimum 16KB recommended)
/// * `identity` - PSK identity
/// * `psk` - Pre-shared key
///
/// # Example
/// ```no_run
/// # #[cfg(feature = "tls")] {
/// use cluster_net::tls::{create_tls_config_with_psk, TLS_BUFFER_SIZE};
///
/// # fn example() {
/// const IDENTITY: &[u8] = b"client01";
/// const PSK: &[u8] = b"secret_key_here";
///
/// let mut rx_buf = [0u8; TLS_BUFFER_SIZE];
/// let mut tx_buf = [0u8; TLS_BUFFER_SIZE];
///
/// let tls = create_tls_config_with_psk(&mut rx_buf, &mut tx_buf, IDENTITY, PSK);
/// # }
/// # }
/// ```
pub fn create_tls_config_with_psk<'a>(
    read_buffer: &'a mut [u8],
    write_buffer: &'a mut [u8],
    identity: &'a [u8],
    psk: &'a [u8],
) -> TlsConfig<'a> {
    TlsConfig::new(
        DEFAULT_TLS_SEED,
        read_buffer,
        write_buffer,
        TlsVerify::Psk { identity, psk },
    )
}
