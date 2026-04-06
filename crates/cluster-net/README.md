# cluster-net

REST API client library for cluster-matrix, providing read-only access to cluster data.

## Features

- **No-std compatible** - Works in embedded environments
- **Async/await** - Built on Embassy and embedded-nal-async
- **HTTP and HTTPS** - Optional TLS support via the `tls` feature
- **JSON parsing** - Uses serde-json-core for no-std JSON deserialization
- **Read-only API** - Fetch cluster data, layouts, and poll for updates

## Usage

### Basic HTTP Example

```rust
use cluster_net::{Client, ClientConfig, Endpoints};
use cluster_core::types::ClusterId;

async fn fetch_cluster_data<T, D>(tcp: &T, dns: &D)
where
    T: embedded_nal_async::TcpConnect,
    D: embedded_nal_async::Dns,
{
    // Create client configuration
    let config = ClientConfig::new("http://api.example.com")
        .unwrap()
        .with_timeout(10000); // 10 second timeout

    // Create HTTP client
    let mut client = Client::new(config, tcp, dns);

    // Fetch cluster data
    let mut buffer = [0u8; 8192];
    let cluster = Endpoints::get_cluster(&mut client, ClusterId::F0, &mut buffer)
        .await
        .unwrap();

    println!("Cluster has {} seats", cluster.seats.len());
}
```

### HTTPS Example (with `tls` feature)

```rust
use cluster_net::{Client, ClientConfig, Endpoints, TlsConfigBuilder};
use cluster_core::types::ClusterId;

// Embed CA certificate at compile time
const CA_CERT: &[u8] = include_bytes!("../certs/root-ca.der");

async fn fetch_cluster_data_https<T, D>(tcp: &T, dns: &D)
where
    T: embedded_nal_async::TcpConnect,
    D: embedded_nal_async::Dns,
{
    // Configure TLS
    let tls = TlsConfigBuilder::new()
        .with_server_name("api.example.com")
        .with_ca_certificate(CA_CERT)
        .build();

    // Create HTTPS client
    let config = ClientConfig::new("https://api.example.com").unwrap();
    let mut client = Client::new_with_tls(config, tcp, dns, tls);

    // Fetch cluster data (same API as HTTP)
    let mut buffer = [0u8; 8192];
    let cluster = Endpoints::get_cluster(&mut client, ClusterId::F0, &mut buffer)
        .await
        .unwrap();

    println!("Cluster has {} seats", cluster.seats.len());
}
```

### Fetching Complete Layout

```rust
use cluster_net::{Client, Endpoints};

async fn fetch_layout<T, D>(client: &mut Client<'_, T, D>)
where
    T: embedded_nal_async::TcpConnect,
    D: embedded_nal_async::Dns,
{
    // Use a larger buffer for the complete layout
    let mut buffer = [0u8; 16384];

    let layout = Endpoints::get_layout(client, &mut buffer)
        .await
        .unwrap();

    println!("Layout contains clusters: f0, f1, f1b, f2, f4, f6");
}
```

### Polling for Updates

```rust
use cluster_net::{Client, Endpoints};
use cluster_core::types::ClusterId;
use embassy_time::{Duration, Timer};

async fn poll_cluster_updates<T, D>(client: &mut Client<'_, T, D>)
where
    T: embedded_nal_async::TcpConnect,
    D: embedded_nal_async::Dns,
{
    let mut buffer = [0u8; 8192];

    loop {
        match Endpoints::poll_cluster(client, ClusterId::F0, &mut buffer).await {
            Ok(cluster) => {
                println!("Occupancy: {}%", cluster.occupancy_percentage());
            }
            Err(e) => {
                eprintln!("Poll error: {:?}", e);
            }
        }

        // Wait 30 seconds before next poll
        Timer::after(Duration::from_secs(30)).await;
    }
}
```

## Feature Flags

- `std` - Enable standard library support (for testing and non-embedded use)
- `defmt` - Enable defmt logging for debugging
- `tls` - Enable HTTPS/TLS support via embedded-tls

## API Endpoints

### `Endpoints::get_cluster(client, cluster_id, buffer) -> Result<Cluster>`

Fetch data for a specific cluster by ID.

**Parameters:**
- `client` - Mutable reference to HTTP/HTTPS client
- `cluster_id` - ClusterId enum (F0, F1, F1b, F2, F4, F6)
- `buffer` - Mutable byte buffer for response (recommended: 8KB+)

**Returns:** `Cluster` struct with seat data, zones, and attributes

### `Endpoints::get_layout(client, buffer) -> Result<Layout>`

Fetch the complete layout containing all clusters.

**Parameters:**
- `client` - Mutable reference to HTTP/HTTPS client
- `buffer` - Mutable byte buffer for response (recommended: 16KB+)

**Returns:** `Layout` struct with all cluster data

### `Endpoints::poll_cluster(client, cluster_id, buffer) -> Result<Cluster>`

Poll for cluster updates. This is an alias for `get_cluster` intended for periodic polling.

## TLS Configuration

### Certificate Formats

The TLS implementation expects certificates in DER format. To convert PEM to DER:

```bash
# Convert PEM certificate to DER
openssl x509 -in cert.pem -outform der -out cert.der
```

### Security Considerations

**Certificate Verification:**
- Embed certificates at compile time using `include_bytes!`

**Server Name Indication (SNI):**
- Always set the server name with `.with_server_name()`
- Must match the certificate's Common Name or SAN

**Memory Requirements:**
- TLS requires larger buffers (~16KB for RX/TX)
- Budget ~32KB+ RAM for TLS operations

## Error Handling

All network operations return `Result<T, cluster_net::Error>`:

```rust
use cluster_net::Error;

match Endpoints::get_cluster(client, ClusterId::F0, buffer).await {
    Ok(cluster) => {
        // Handle cluster data
    }
    Err(Error::ConnectionError) => {
        // Network connection failed
    }
    Err(Error::InvalidStatus(code)) => {
        // HTTP error (e.g., 404, 500)
    }
    Err(Error::DeserializationError) => {
        // JSON parsing failed
    }
    Err(e) => {
        // Other errors
    }
}
```

## Dependencies

- `reqwless` - Embedded HTTP client
- `embedded-nal-async` - Network abstraction layer
- `serde-json-core` - No-std JSON parsing
- `heapless` - Stack-allocated data structures
- `cluster-core` - Cluster data models
- `embedded-tls` (optional) - TLS 1.3 implementation
- `rand` (optional) - Random number generation for TLS
