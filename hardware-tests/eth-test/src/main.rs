//! cluster-net test on RP2350 with WIZnet W6100 ethernet
//!
//! This example tests the cluster-net library on embedded hardware,
//! demonstrating HTTP (and optionally HTTPS) requests to fetch cluster data.
//!
//! Hardware configuration:
//! - WIZnet W6100 ethernet chip
//! - Pin mapping: MISO=16, MOSI=19, SCLK=18, CSn=17, RSTn=20, INTn=21

#![no_std]
#![no_main]

mod compat;

use crate::compat::StackAdapter;
use cluster_core::types::ClusterId;
use cluster_net::client::{Client, ClientConfig};
use cluster_net::endpoints::Endpoints;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_net::{Stack, StackResources};
use embassy_net_wiznet::chip::W6100;
use embassy_net_wiznet::{Device, Runner, State};
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::SPI0;
use embassy_rp::spi::{Async, Config as SpiConfig, Spi};
use embassy_time::{Delay, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Test configuration
const TEST_SERVER_URL: &str = "http://example.com"; // Replace with your test server
const TEST_INTERVAL_SECS: u64 = 30;

#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<
        'static,
        W6100,
        ExclusiveDevice<Spi<'static, SPI0, Async>, Output<'static>, Delay>,
        Input<'static>,
        Output<'static>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting cluster-net hardware test on RP2350 + W6100");

    let p = embassy_rp::init(Default::default());
    let mut rng = RoscRng;

    // W6100 SPI configuration
    info!("Configuring W6100 ethernet...");
    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = 50_000_000;

    // Pin mapping: MISO=16, MOSI=19, SCLK=18, CSn=17, RSTn=20, INTn=21
    let (miso, mosi, clk) = (p.PIN_16, p.PIN_19, p.PIN_18);
    let spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, spi_cfg);
    let cs = Output::new(p.PIN_17, Level::High);
    let w6100_int = Input::new(p.PIN_21, Pull::Up);
    let w6100_reset = Output::new(p.PIN_20, Level::High);

    let mac_addr = [0x02, 0x00, 0x00, 0x00, 0x00, 0x01];
    static STATE: StaticCell<State<8, 8>> = StaticCell::new();
    let state = STATE.init(State::<8, 8>::new());

    let spi_dev = ExclusiveDevice::new(spi, cs, Delay).unwrap();

    let (device, runner) =
        embassy_net_wiznet::new(mac_addr, state, spi_dev, w6100_int, w6100_reset)
            .await
            .unwrap();

    spawner.spawn(unwrap!(ethernet_task(runner)));

    // Generate random seed for network stack
    let seed = rng.next_u64();

    // Init network stack with DHCP
    info!("Initializing network stack...");
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        device,
        embassy_net::Config::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::new()),
        seed,
    );

    // Launch network task
    spawner.spawn(unwrap!(net_task(runner)));

    // Wait for network configuration
    info!("Waiting for DHCP...");
    let cfg = wait_for_config(stack).await;
    info!("Network configured!");
    info!("  IP address:  {:?}", cfg.address.address());
    info!("  Gateway:     {:?}", cfg.gateway);
    info!("  DNS servers: {:?}", cfg.dns_servers);

    // Wait a bit for network to stabilize
    Timer::after_secs(2).await;

    // Run HTTP tests
    info!("Starting HTTP tests...");
    test_http_client(stack).await;

    // Optional: Run HTTPS tests if TLS feature is enabled
    #[cfg(feature = "tls")]
    {
        info!("Starting HTTPS tests...");
        test_https_client(stack).await;
    }

    // Continuous polling loop
    info!(
        "Entering continuous polling mode (every {} seconds)",
        TEST_INTERVAL_SECS
    );
    loop {
        Timer::after_secs(TEST_INTERVAL_SECS).await;

        match poll_cluster_data(stack).await {
            Ok(()) => info!("Poll successful"),
            Err(e) => error!("Poll failed: {:?}", e),
        }
    }
}

/// Wait for network configuration from DHCP
async fn wait_for_config(stack: Stack<'static>) -> embassy_net::StaticConfigV4 {
    loop {
        if let Some(config) = stack.config_v4() {
            return config.clone();
        }
        yield_now().await;
    }
}

/// Test HTTP client functionality
async fn test_http_client(stack: Stack<'static>) {
    info!("=== HTTP Client Test ===");

    // Create client configuration
    let config = match ClientConfig::new(TEST_SERVER_URL) {
        Ok(cfg) => cfg.with_timeout(10000),
        Err(_) => {
            error!("Failed to create client config (URL too long?)");
            return;
        }
    };

    // Create compatibility adapter for embassy-net stack
    let adapter = StackAdapter::new(&stack);

    // Create HTTP client using the adapter
    let mut client: Client<StackAdapter, StackAdapter> = Client::new(config, &adapter, &adapter);

    // Test 1: Fetch cluster F0
    info!("Test 1: Fetching cluster F0...");
    let mut buffer = [0u8; 8192];

    // Scope the first borrow explicitly
    {
        match Endpoints::get_cluster(&mut client, ClusterId::F0, &mut buffer).await {
            Ok(cluster) => {
                info!("✓ Successfully fetched cluster F0");
                info!("  Name: {}", cluster.name.as_str());
                info!("  Seats: {}", cluster.seats.len());
                info!("  Zones: {}", cluster.zones.len());
                info!("  Occupancy: {}%", cluster.occupancy_percentage());

                let stats = cluster.get_stats();
                info!(
                    "  Stats: total={}, available={}, occupied={}, broken={}",
                    stats.total, stats.available, stats.occupied, stats.out_of_order
                );
            }
            Err(e) => {
                error!("✗ Failed to fetch cluster: {:?}", e);
            }
        }
    } // First borrow of client ends here

    // Small delay between requests
    Timer::after_millis(500).await;

    // Test 2: Fetch complete layout
    info!("Test 2: Fetching complete layout...");
    let mut large_buffer = [0u8; 16384]; // Larger buffer for layout

    match Endpoints::get_layout(&mut client, &mut large_buffer).await {
        Ok(layout) => {
            info!("✓ Successfully fetched layout");
            info!("  F0 seats: {}", layout.f0.seats.len());
            info!("  F1 seats: {}", layout.f1.seats.len());
            info!("  F2 seats: {}", layout.f2.seats.len());
            info!("  F4 seats: {}", layout.f4.seats.len());
            info!("  F6 seats: {}", layout.f6.seats.len());
        }
        Err(e) => {
            error!("✗ Failed to fetch layout: {:?}", e);
        }
    }

    info!("=== HTTP Test Complete ===");
}

/// Test HTTPS client functionality (only with TLS feature)
#[cfg(feature = "tls")]
async fn test_https_client(stack: Stack<'static>) {
    use cluster_net::tls::{create_tls_config, TLS_BUFFER_SIZE};

    info!("=== HTTPS Client Test ===");

    // Allocate TLS buffers
    let mut rx_buffer = [0u8; TLS_BUFFER_SIZE];
    let mut tx_buffer = [0u8; TLS_BUFFER_SIZE];

    // Create TLS config (no verification for testing)
    let tls = create_tls_config(&mut rx_buffer, &mut tx_buffer);

    // Create HTTPS client configuration
    let config = match ClientConfig::new("https://example.com") {
        Ok(cfg) => cfg.with_timeout(10000),
        Err(_) => {
            error!("Failed to create HTTPS client config");
            return;
        }
    };

    // Create compatibility adapter for embassy-net stack
    let adapter = compat::StackAdapter::new(&stack);

    // Create HTTPS client
    let mut client = Client::new_with_tls(config, &adapter, &adapter, tls);

    // Test HTTPS request
    info!("Test: Fetching cluster via HTTPS...");
    let mut buffer = [0u8; 8192];

    match Endpoints::get_cluster(&mut client, ClusterId::F0, &mut buffer).await {
        Ok(cluster) => {
            info!("✓ Successfully fetched cluster via HTTPS");
            info!("  Name: {}", cluster.name.as_str());
            info!("  Seats: {}", cluster.seats.len());
        }
        Err(e) => {
            error!("✗ Failed to fetch via HTTPS: {:?}", e);
        }
    }

    info!("=== HTTPS Test Complete ===");
}

/// Poll cluster data periodically
async fn poll_cluster_data(stack: Stack<'static>) -> Result<(), ()> {
    let config = ClientConfig::new(TEST_SERVER_URL).map_err(|_| ())?;
    let adapter = StackAdapter::new(&stack);
    let mut client: Client<StackAdapter, StackAdapter> = Client::new(config, &adapter, &adapter);

    let mut buffer = [0u8; 8192];
    let cluster = Endpoints::poll_cluster(&mut client, ClusterId::F0, &mut buffer)
        .await
        .map_err(|_| ())?;

    info!(
        "Cluster F0 update: {} seats, {}% occupied",
        cluster.seats.len(),
        cluster.occupancy_percentage()
    );

    Ok(())
}
