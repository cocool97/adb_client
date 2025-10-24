use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;

use super::DeviceCommands;

#[derive(Parser, Debug)]
pub struct TcpCommand {
    pub address: SocketAddr,
    /// Path to a custom private key to use for authentication
    #[clap(short = 'k', long = "private-key")]
    pub path_to_private_key: Option<PathBuf>,
    #[clap(subcommand)]
    pub commands: DeviceCommands,
}
