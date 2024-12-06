use clap::Parser;
use std::net::SocketAddr;

use super::DeviceCommands;

#[derive(Parser, Debug)]
pub struct TcpCommand {
    pub address: SocketAddr,
    #[clap(subcommand)]
    pub commands: DeviceCommands,
}
