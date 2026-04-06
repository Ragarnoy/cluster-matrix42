//! REST API endpoints for cluster data

use crate::client::Client;
use crate::error::{Error, Result};
use cluster_core::models::{Cluster, Layout};
use cluster_core::types::ClusterId;
use embedded_nal_async::{Dns, TcpConnect};
use heapless::String;

/// API endpoints namespace
pub struct Endpoints;

impl Endpoints {
    /// Get cluster data by ID
    ///
    /// # Arguments
    /// * `client` - HTTP client instance
    /// * `cluster_id` - The cluster ID to fetch
    /// * `buffer` - Buffer for HTTP response
    ///
    /// # Example
    /// ```no_run
    /// # use cluster_net::endpoints::Endpoints;
    /// # use cluster_net::client::{Client, ClientConfig};
    /// # use cluster_core::types::ClusterId;
    /// # async fn example<T: embedded_nal_async::TcpConnect, D: embedded_nal_async::Dns>(client: &mut Client<'_, T, D>) {
    /// let mut buffer = [0u8; 8192];
    /// let cluster = Endpoints::get_cluster(client, ClusterId::F0, &mut buffer).await.unwrap();
    /// # }
    /// ```
    pub async fn get_cluster<'c, 'a, T: TcpConnect, D: Dns, const BUF_SIZE: usize>(
        client: &'c mut Client<'a, T, D, BUF_SIZE>,
        cluster_id: ClusterId,
        buffer: &mut [u8],
    ) -> Result<Cluster> {
        use core::fmt::Write;

        // Construct path
        let mut path: String<64> = String::new();
        path.push_str("/cluster/").map_err(|_| Error::InvalidUrl)?;
        write!(&mut path, "{}", cluster_id).map_err(|_| Error::InvalidUrl)?;

        // Make request
        let response_body = client.get(path.as_str(), buffer).await?;

        // Parse JSON response
        let (cluster, _) = serde_json_core::from_slice::<Cluster>(response_body)
            .map_err(|_| Error::DeserializationError)?;

        #[cfg(feature = "defmt")]
        defmt::debug!(
            "Fetched cluster: {} with {} seats",
            cluster.name.as_str(),
            cluster.seats.len()
        );

        Ok(cluster)
    }

    /// Get complete layout with all clusters
    ///
    /// # Arguments
    /// * `client` - HTTP client instance
    /// * `buffer` - Buffer for HTTP response (should be large enough for the entire layout)
    ///
    /// # Example
    /// ```no_run
    /// # use cluster_net::endpoints::Endpoints;
    /// # use cluster_net::client::{Client, ClientConfig};
    /// # async fn example<T: embedded_nal_async::TcpConnect, D: embedded_nal_async::Dns>(client: &mut Client<'_, T, D>) {
    /// let mut buffer = [0u8; 16384]; // Larger buffer for complete layout
    /// let layout = Endpoints::get_layout(client, &mut buffer).await.unwrap();
    /// # }
    /// ```
    pub async fn get_layout<'c, 'a, T: TcpConnect, D: Dns, const BUF_SIZE: usize>(
        client: &'c mut Client<'a, T, D, BUF_SIZE>,
        buffer: &mut [u8],
    ) -> Result<Layout> {
        // Make request
        let response_body = client.get("/layout", buffer).await?;

        // Parse JSON response
        let (layout, _) = serde_json_core::from_slice::<Layout>(response_body)
            .map_err(|_| Error::DeserializationError)?;

        #[cfg(feature = "defmt")]
        defmt::debug!("Fetched complete layout");

        Ok(layout)
    }

    /// Poll for cluster updates
    ///
    /// This endpoint can be called periodically to fetch updated cluster data.
    ///
    /// # Arguments
    /// * `client` - HTTP client instance
    /// * `cluster_id` - The cluster ID to poll
    /// * `buffer` - Buffer for HTTP response
    pub async fn poll_cluster<'c, 'a, T: TcpConnect, D: Dns, const BUF_SIZE: usize>(
        client: &'c mut Client<'a, T, D, BUF_SIZE>,
        cluster_id: ClusterId,
        buffer: &mut [u8],
    ) -> Result<Cluster> {
        // Reuse get_cluster for polling
        Self::get_cluster(client, cluster_id, buffer).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_construction() {
        let mut path: String<64> = String::new();
        path.push_str("/cluster/").unwrap();
        path.push_str("f0").unwrap();
        assert_eq!(path.as_str(), "/cluster/f0");
    }
}
