use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, info};

use crate::config::NetworkConfig;
use crate::error::{LostLoveError, Result};

/// TUN/TAP interface wrapper
pub struct TunInterface {
    device: tun::AsyncDevice,
    name: String,
    mtu: usize,
}

impl TunInterface {
    /// Create new TUN interface
    pub async fn new(config: &NetworkConfig) -> Result<Self> {
        info!("Creating TUN interface: {}", config.tun_name);

        let mut tun_config = tun::Configuration::default();

        tun_config
            .name(&config.tun_name)
            .mtu(config.mtu as i32)
            .up();

        // Parse IP address and netmask
        let (ip, netmask) = parse_cidr(&config.tun_address)
            .map_err(|e| LostLoveError::Network(format!("Invalid tun_address: {}", e)))?;

        #[cfg(target_os = "linux")]
        {
            tun_config.address(ip).netmask(netmask);
        }

        #[cfg(target_os = "macos")]
        {
            tun_config.address(ip).destination(netmask);
        }

        #[cfg(target_os = "windows")]
        {
            tun_config.address(ip).netmask(netmask);
        }

        let device = tun::create_as_async(&tun_config)
            .map_err(|e| LostLoveError::Network(format!("Failed to create TUN device: {}", e)))?;

        info!(
            "TUN interface {} created successfully (MTU: {})",
            config.tun_name, config.mtu
        );

        Ok(Self {
            device,
            name: config.tun_name.clone(),
            mtu: config.mtu,
        })
    }

    /// Get interface name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get MTU
    pub fn mtu(&self) -> usize {
        self.mtu
    }

    /// Read packet from TUN interface
    pub async fn read_packet(&mut self) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; self.mtu + 4]; // +4 for TUN header on some platforms

        match self.device.read(&mut buf).await {
            Ok(n) => {
                debug!("Read {} bytes from TUN interface", n);
                buf.truncate(n);
                Ok(buf)
            }
            Err(e) => {
                error!("Failed to read from TUN interface: {}", e);
                Err(LostLoveError::from(e))
            }
        }
    }

    /// Write packet to TUN interface
    pub async fn write_packet(&mut self, packet: &[u8]) -> Result<()> {
        if packet.len() > self.mtu {
            return Err(LostLoveError::Network(format!(
                "Packet size {} exceeds MTU {}",
                packet.len(),
                self.mtu
            )));
        }

        match self.device.write_all(packet).await {
            Ok(_) => {
                debug!("Wrote {} bytes to TUN interface", packet.len());
                Ok(())
            }
            Err(e) => {
                error!("Failed to write to TUN interface: {}", e);
                Err(LostLoveError::from(e))
            }
        }
    }

    /// Shutdown the interface
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down TUN interface: {}", self.name);
        Ok(())
    }
}

/// Parse CIDR notation (e.g., "10.8.0.1/24")
fn parse_cidr(cidr: &str) -> io::Result<(std::net::Ipv4Addr, std::net::Ipv4Addr)> {
    let parts: Vec<&str> = cidr.split('/').collect();

    if parts.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid CIDR format",
        ));
    }

    let ip: std::net::Ipv4Addr = parts[0]
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid IP address"))?;

    let prefix_len: u8 = parts[1]
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid prefix length"))?;

    if prefix_len > 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Prefix length must be <= 32",
        ));
    }

    // Calculate netmask from prefix length
    let mask = if prefix_len == 0 {
        0
    } else {
        !0u32 << (32 - prefix_len)
    };

    let netmask = std::net::Ipv4Addr::from(mask);

    Ok((ip, netmask))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cidr() {
        let (ip, netmask) = parse_cidr("10.8.0.1/24").unwrap();
        assert_eq!(ip, "10.8.0.1".parse::<std::net::Ipv4Addr>().unwrap());
        assert_eq!(netmask, "255.255.255.0".parse::<std::net::Ipv4Addr>().unwrap());

        let (ip, netmask) = parse_cidr("192.168.1.1/16").unwrap();
        assert_eq!(ip, "192.168.1.1".parse::<std::net::Ipv4Addr>().unwrap());
        assert_eq!(netmask, "255.255.0.0".parse::<std::net::Ipv4Addr>().unwrap());
    }

    #[test]
    fn test_invalid_cidr() {
        assert!(parse_cidr("10.8.0.1").is_err());
        assert!(parse_cidr("invalid/24").is_err());
        assert!(parse_cidr("10.8.0.1/33").is_err());
    }
}
