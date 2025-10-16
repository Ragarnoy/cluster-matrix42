//! HTTP client implementation

use crate::error::{Error, Result};
use embedded_nal_async::{Dns, TcpConnect};
use heapless::String;
use reqwless::client::HttpClient;
use reqwless::request::{Method, RequestBuilder};

#[cfg(feature = "tls")]
use reqwless::client::TlsConfig;

/// Configuration for the cluster API client
#[derive(Debug, Clone)]
pub struct ClientConfig<const URL_LEN: usize = 128> {
    /// Base URL of the cluster API server
    pub base_url: String<URL_LEN>,
    /// Request timeout in milliseconds
    pub timeout_ms: u32,
}

impl<const URL_LEN: usize> ClientConfig<URL_LEN> {
    /// Create a new client configuration
    pub fn new(base_url: &str) -> core::result::Result<Self, ()> {
        Ok(Self {
            base_url: String::try_from(base_url).map_err(|_| ())?,
            timeout_ms: 5000, // 5 second default timeout
        })
    }

    /// Set the request timeout
    pub fn with_timeout(mut self, timeout_ms: u32) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

/// HTTP client for cluster API
pub struct Client<'a, T: TcpConnect, D: Dns, const BUF_SIZE: usize = 8192> {
    config: ClientConfig,
    http_client: HttpClient<'a, T, D>,
}

impl<'a, T: TcpConnect, D: Dns, const BUF_SIZE: usize> Client<'a, T, D, BUF_SIZE> {
    /// Create a new HTTP client (without TLS)
    ///
    /// # Arguments
    /// * `config` - Client configuration
    /// * `tcp` - TCP connection implementation
    /// * `dns` - DNS resolver implementation
    pub fn new(config: ClientConfig, tcp: &'a T, dns: &'a D) -> Self {
        Self {
            config,
            http_client: HttpClient::new(tcp, dns),
        }
    }

    /// Create a new HTTPS client with TLS support
    ///
    /// # Arguments
    /// * `config` - Client configuration (should use "https://" URLs)
    /// * `tcp` - TCP connection implementation
    /// * `dns` - DNS resolver implementation
    /// * `tls_config` - TLS configuration from TlsConfigBuilder
    ///
    /// # Example
    /// ```no_run
    /// # #[cfg(feature = "tls")] {
    /// use cluster_net::client::{Client, ClientConfig};
    /// use cluster_net::tls::{create_tls_config, TLS_BUFFER_SIZE};
    /// # async fn example<T: embedded_nal_async::TcpConnect, D: embedded_nal_async::Dns>(
    /// #     tcp: &T, dns: &D
    /// # ) {
    /// let mut rx_buf = [0u8; TLS_BUFFER_SIZE];
    /// let mut tx_buf = [0u8; TLS_BUFFER_SIZE];
    /// let tls = create_tls_config(&mut rx_buf, &mut tx_buf);
    ///
    /// let config = ClientConfig::new("https://api.example.com").unwrap();
    /// let mut client = Client::new_with_tls(config, tcp, dns, tls);
    /// # }
    /// # }
    /// ```
    #[cfg(feature = "tls")]
    pub fn new_with_tls(
        config: ClientConfig,
        tcp: &'a T,
        dns: &'a D,
        tls_config: TlsConfig<'a>,
    ) -> Self {
        Self {
            config,
            http_client: HttpClient::new_with_tls(tcp, dns, tls_config),
        }
    }

    /// Perform a GET request to the specified path
    ///
    /// # Arguments
    /// * `path` - The API path to request (e.g., "/cluster/f0")
    /// * `buffer` - Buffer to store the response body
    ///
    /// # Returns
    /// The number of bytes read into the buffer
    pub async fn get<'buf>(&mut self, path: &str, buffer: &'buf mut [u8]) -> Result<&'buf [u8]> {
        // Construct full URL
        let mut url: String<{ crate::MAX_URL_LENGTH }> = String::new();
        url.push_str(self.config.base_url.as_str())
            .map_err(|_| Error::InvalidUrl)?;
        url.push_str(path).map_err(|_| Error::InvalidUrl)?;

        #[cfg(feature = "defmt")]
        defmt::debug!("GET {}", url.as_str());

        // Create request
        let request = self
            .http_client
            .request(Method::GET, url.as_str())
            .await
            .map_err(|_| Error::HttpError)?;

        // Add common headers
        let headers = [("Accept", "application/json")];
        let mut request_with_headers = request.headers(&headers);

        // Send request and get response
        let response = request_with_headers
            .send(buffer)
            .await
            .map_err(|_| Error::ConnectionError)?;

        // Check status code
        let status = response.status;
        if !(200..300).contains(&(status.0)) {
            #[cfg(feature = "defmt")]
            defmt::error!("HTTP error: status {}", status.0);
            return Err(Error::InvalidStatus(status.0));
        }

        // Read response body
        let body = response
            .body()
            .read_to_end()
            .await
            .map_err(|_| Error::HttpError)?;

        #[cfg(feature = "defmt")]
        defmt::debug!("Response: {} bytes", body.len());

        Ok(body)
    }

    /// Get the client configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_creation() {
        let config = ClientConfig::new("http://example.com").unwrap();
        assert_eq!(config.base_url.as_str(), "http://example.com");
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_client_config_with_timeout() {
        let config = ClientConfig::new("http://example.com")
            .unwrap()
            .with_timeout(10000);
        assert_eq!(config.timeout_ms, 10000);
    }
}
