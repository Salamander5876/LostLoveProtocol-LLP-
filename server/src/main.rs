use anyhow::Result;
use clap::Parser;
use tracing::{info, error};
use tracing_subscriber;

mod protocol;
mod core;
mod network;
mod config;
mod error;

use crate::core::server::Server;
use crate::config::Config;

/// LostLove Protocol VPN Server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "/etc/lostlove/server.toml")]
    config: String,

    /// Check configuration and exit
    #[arg(long)]
    check_config: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = args.log_level.parse().unwrap_or(tracing::Level::INFO);
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(true)
        .init();

    info!("LostLove Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Loading configuration from: {}", args.config);

    // Load configuration
    let config = Config::load(&args.config)?;

    if args.check_config {
        info!("Configuration is valid!");
        return Ok(());
    }

    // Create and start server
    let server = Server::new(config).await?;

    info!("Starting server...");

    // Run server
    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
        return Err(e);
    }

    Ok(())
}
