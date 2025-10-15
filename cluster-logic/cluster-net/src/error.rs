//! Error types for network operations

use core::fmt;

/// Errors that can occur during network operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// HTTP request failed
    HttpError,
    /// Response parsing failed
    ParseError,
    /// Invalid response status code
    InvalidStatus(u16),
    /// Deserialization failed
    DeserializationError,
    /// Buffer too small for operation
    BufferTooSmall,
    /// Network connection error
    ConnectionError,
    /// Request timeout
    Timeout,
    /// Invalid URL format
    InvalidUrl,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::HttpError => write!(f, "HTTP request failed"),
            Error::ParseError => write!(f, "Response parsing failed"),
            Error::InvalidStatus(code) => write!(f, "Invalid HTTP status: {}", code),
            Error::DeserializationError => write!(f, "JSON deserialization failed"),
            Error::BufferTooSmall => write!(f, "Buffer too small"),
            Error::ConnectionError => write!(f, "Network connection error"),
            Error::Timeout => write!(f, "Request timeout"),
            Error::InvalidUrl => write!(f, "Invalid URL format"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(feature = "defmt")]
impl defmt::Format for Error {
    fn format(&self, f: defmt::Formatter) {
        match self {
            Error::HttpError => defmt::write!(f, "HTTP request failed"),
            Error::ParseError => defmt::write!(f, "Response parsing failed"),
            Error::InvalidStatus(code) => defmt::write!(f, "Invalid HTTP status: {}", code),
            Error::DeserializationError => defmt::write!(f, "JSON deserialization failed"),
            Error::BufferTooSmall => defmt::write!(f, "Buffer too small"),
            Error::ConnectionError => defmt::write!(f, "Network connection error"),
            Error::Timeout => defmt::write!(f, "Request timeout"),
            Error::InvalidUrl => defmt::write!(f, "Invalid URL format"),
        }
    }
}

/// Result type for network operations
pub type Result<T> = core::result::Result<T, Error>;
