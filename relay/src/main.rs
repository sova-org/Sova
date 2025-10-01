use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use tracing::{info, warn};

mod relay;
mod types;
mod web;

use relay::RelayServer;

/// Sova Relay Server for remote collaboration
#[derive(Parser)]
#[command(name = "sova-relay")]
#[command(about = "A relay server for remote Sova collaboration")]
struct Args {
    /// IP address to bind to
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,

    /// Port to listen on for relay connections
    #[arg(short, long, default_value_t = 9090)]
    port: u16,

    /// Port to listen on for HTTP web interface
    #[arg(long, default_value_t = 8080)]
    http_port: u16,

    /// Maximum number of concurrent instances
    #[arg(long, default_value_t = 20)]
    max_instances: usize,

    /// Maximum message rate per instance (messages per minute)
    #[arg(long, default_value_t = 1000)]
    rate_limit: u32,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(&args.log_level)
        .init();

    info!("Starting Sova Relay Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Relay listening on {}:{}", args.host, args.port);
    info!("Web interface listening on {}:{}", args.host, args.http_port);
    info!("Max instances: {}", args.max_instances);
    info!("Rate limit: {} msg/min per instance", args.rate_limit);

    let relay_addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let http_addr: SocketAddr = format!("{}:{}", args.host, args.http_port).parse()?;
    
    let server = RelayServer::new(args.max_instances, args.rate_limit);
    
    // Handle shutdown gracefully
    tokio::select! {
        result = server.run(relay_addr, http_addr) => {
            if let Err(e) = result {
                warn!("Server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down gracefully...");
        }
    }

    info!("Relay server stopped");
    Ok(())
}