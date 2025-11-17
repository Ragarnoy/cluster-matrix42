//! Compatibility layer for embedded-nal-async 0.8 traits with embassy-net
//!
//! This module provides adapters that implement embedded-nal-async 0.8 traits
//! for embassy-net Stack, allowing reqwless 0.13 to work with embassy-net.
//!
//! Note: This adapter can only handle one connection at a time, which is
//! sufficient for reqwless's usage pattern.

use core::cell::UnsafeCell;
use core::fmt::Write;
use core::net::{IpAddr, SocketAddr};
#[cfg(not(feature = "defmt"))]
use embassy_net::tcp::ConnectError;
use embassy_net::tcp::Error;
use embassy_net::{Stack, dns::DnsQueryType};
use embedded_nal_async_08::{Dns, TcpConnect};

pub const TCP_RX_BUFFER_SIZE: usize = 4096;
pub const TCP_TX_BUFFER_SIZE: usize = 4096;

// Convert embassy-net IpAddress to core::net::IpAddr
// This is a workaround for the type conversion between smoltcp and core::net types
fn convert_ip_addr(addr: embassy_net::IpAddress) -> Result<IpAddr, embassy_net::dns::Error> {
    use core::str::FromStr;
    use heapless::String;

    // Format the IP address into a string
    // IPv6 addresses can be up to 45 characters: "ffff:ffff:ffff:ffff:ffff:ffff:255.255.255.255"
    let mut ip_str: String<46> = String::new();
    write!(&mut ip_str, "{}", addr).map_err(|_| embassy_net::dns::Error::Failed)?;

    // Parse it back as core::net::IpAddr
    IpAddr::from_str(ip_str.as_str()).map_err(|_| embassy_net::dns::Error::Failed)
}

/// Compatibility adapter for embassy-net Stack with buffer storage
///
/// This adapter stores TCP socket buffers and can only handle one connection
/// at a time. This is safe for reqwless which only maintains one connection.
pub struct StackAdapter<'a> {
    stack: &'a Stack<'a>,
    rx_buffer: UnsafeCell<[u8; TCP_RX_BUFFER_SIZE]>,
    tx_buffer: UnsafeCell<[u8; TCP_TX_BUFFER_SIZE]>,
}

/// Safety: The adapter is designed for single-threaded embassy executor
/// and reqwless only creates one connection at a time
unsafe impl<'a> Sync for StackAdapter<'a> {}

impl<'a> StackAdapter<'a> {
    pub fn new(stack: &'a Stack<'a>) -> Self {
        Self {
            stack,
            rx_buffer: UnsafeCell::new([0; TCP_RX_BUFFER_SIZE]),
            tx_buffer: UnsafeCell::new([0; TCP_TX_BUFFER_SIZE]),
        }
    }
}

// No conversion needed - embedded-nal-async 0.8 uses core::net types directly

// Implement TcpConnect trait from embedded-nal-async 0.8
impl<'a> TcpConnect for StackAdapter<'a> {
    type Error = Error;
    type Connection<'m>
        = embassy_net::tcp::TcpSocket<'m>
    where
        Self: 'm;

    async fn connect<'m>(
        &'m self,
        remote: SocketAddr,
    ) -> Result<Self::Connection<'m>, Self::Error> {
        // Safety: We're getting mutable access to the buffers stored in UnsafeCells.
        // This is safe because:
        // 1. reqwless only creates one connection at a time
        // 2. embassy executor is single-threaded
        // 3. The returned socket borrows from 'm which ties it to &'m self
        let rx_buf = unsafe { &mut *self.rx_buffer.get() };
        let tx_buf = unsafe { &mut *self.tx_buffer.get() };

        let mut socket = embassy_net::tcp::TcpSocket::new(*self.stack, rx_buf, tx_buf);

        // Convert SocketAddr to IpEndpoint (embassy-net uses IpEndpoint internally)
        let endpoint = match remote {
            SocketAddr::V4(addr) => (*addr.ip(), addr.port()),
            SocketAddr::V6(_) => return Err(Error::ConnectionReset), // IPv6 not supported in this path
        };

        socket.connect(endpoint).await.map_err(|e| {
            #[cfg(feature = "defmt")]
            {
                defmt::warn!("Connection error: {:?}", e);
                Error::ConnectionReset
            }
            #[cfg(not(feature = "defmt"))]
            match e {
                ConnectError::InvalidState => Error::ConnectionReset,
                ConnectError::NoRoute => Error::ConnectionReset,
                ConnectError::ConnectionReset => Error::ConnectionReset,
                ConnectError::TimedOut => Error::ConnectionReset,
            }
        })?;
        Ok(socket)
    }
}

// Implement Dns trait from embedded-nal-async 0.8
impl<'a> Dns for StackAdapter<'a> {
    type Error = embassy_net::dns::Error;

    async fn get_host_by_name(
        &self,
        host: &str,
        addr_type: embedded_nal_async_08::AddrType,
    ) -> Result<IpAddr, Self::Error> {
        // Convert addr_type to DnsQueryType
        let query_type = match addr_type {
            embedded_nal_async_08::AddrType::IPv4 => DnsQueryType::A,
            embedded_nal_async_08::AddrType::IPv6 => DnsQueryType::Aaaa,
            _ => DnsQueryType::A, // Default to IPv4
        };

        let addr = self.stack.dns_query(host, query_type).await?;
        let ip = addr.first().ok_or(embassy_net::dns::Error::Failed)?;
        convert_ip_addr(*ip)
    }

    async fn get_host_by_address(
        &self,
        _addr: IpAddr,
        _result: &mut [u8],
    ) -> Result<usize, Self::Error> {
        // Reverse DNS is not commonly supported in embedded systems
        // Return an error indicating it's not supported
        Err(embassy_net::dns::Error::Failed)
    }
}
