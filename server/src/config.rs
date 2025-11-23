use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub network: NetworkConfig,
    #[serde(default)]
    pub limits: LimitsConfig,
    #[serde(default)]
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_bind_address")]
    pub bind_address: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_protocol")]
    pub protocol: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    #[serde(default = "default_worker_threads")]
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkConfig {
    #[serde(default = "default_tun_name")]
    pub tun_name: String,

    #[serde(default = "default_tun_address")]
    pub tun_address: String,

    #[serde(default = "default_mtu")]
    pub mtu: usize,

    #[serde(default)]
    pub enable_ipv6: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitsConfig {
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_user: u64,

    #[serde(default = "default_max_streams")]
    pub max_streams_per_connection: usize,

    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    #[serde(default = "default_true")]
    pub enable_metrics: bool,

    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,

    #[serde(default = "default_log_level")]
    pub log_level: String,
}

// Defaults
fn default_bind_address() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 8443 }
fn default_protocol() -> String { "tcp".to_string() }
fn default_max_connections() -> usize { 1000 }
fn default_worker_threads() -> usize { 0 }
fn default_tun_name() -> String { "hfp0".to_string() }
fn default_tun_address() -> String { "10.8.0.1/24".to_string() }
fn default_mtu() -> usize { 1400 }
fn default_rate_limit() -> u64 { 100_000_000 }
fn default_max_streams() -> usize { 256 }
fn default_connection_timeout() -> u64 { 300 }
fn default_true() -> bool { true }
fn default_metrics_port() -> u16 { 9090 }
fn default_log_level() -> String { "info".to_string() }

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            rate_limit_per_user: default_rate_limit(),
            max_streams_per_connection: default_max_streams(),
            connection_timeout: default_connection_timeout(),
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: default_true(),
            metrics_port: default_metrics_port(),
            log_level: default_log_level(),
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .context("Failed to read configuration file")?;

        let config: Config = toml::from_str(&content)
            .context("Failed to parse configuration file")?;

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        // Validate bind address
        if self.server.bind_address.is_empty() {
            anyhow::bail!("bind_address cannot be empty");
        }

        // Validate port
        if self.server.port == 0 {
            anyhow::bail!("port must be greater than 0");
        }

        // Validate protocol
        if !["tcp", "udp", "both"].contains(&self.server.protocol.as_str()) {
            anyhow::bail!("protocol must be one of: tcp, udp, both");
        }

        // Validate MTU
        if self.network.mtu < 576 || self.network.mtu > 9000 {
            anyhow::bail!("MTU must be between 576 and 9000");
        }

        Ok(())
    }

    pub fn default_for_testing() -> Self {
        Self {
            server: ServerConfig {
                bind_address: "127.0.0.1".to_string(),
                port: 8443,
                protocol: "tcp".to_string(),
                max_connections: 100,
                worker_threads: 2,
            },
            network: NetworkConfig {
                tun_name: "hfp0".to_string(),
                tun_address: "10.8.0.1/24".to_string(),
                mtu: 1400,
                enable_ipv6: false,
            },
            limits: LimitsConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default_for_testing();
        assert_eq!(config.server.port, 8443);
        assert_eq!(config.network.mtu, 1400);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default_for_testing();
        assert!(config.validate().is_ok());

        // Test invalid MTU
        config.network.mtu = 100;
        assert!(config.validate().is_err());
    }
}
