use clap::{Parser, command};
use std::net::SocketAddrV4;

#[derive(Debug, Parser)]
#[command(version, about = "Simple Rust-based IP filter of incoming connections")]
pub(crate) struct Args {
    /// Listen address
    #[arg(short, long)]
    pub listen: SocketAddrV4,
    /// Dest address
    #[arg(short, long)]
    pub dest: SocketAddrV4,
    /// External service API Key (ProxyCheck)
    #[arg(short, long)]
    pub api_key: String,
    /// Boolean flag indicates will the filter save IP
    #[arg(short, long)]
    pub persistent: bool,
    /// Mount-path for .json logs file
    #[arg(short, long)]
    pub mount_path: Option<String>,
    /// External API Poll period (seconds). By default 15 minutes
    #[arg(short, long, default_value = "10")]
    pub poll_period: u16,
    /// Path to file with list of excluded IP range (CIDR notation)
    #[arg(short, long)]
    pub exluded_ip: Option<String>,
}
