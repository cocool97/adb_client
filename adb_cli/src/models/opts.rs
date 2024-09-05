use std::net::SocketAddrV4;

use clap::Parser;

use crate::commands::{EmuCommand, HostCommand, LocalCommand};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Opts {
    #[clap(short = 'v', long = "verbose")]
    pub verbose: bool,
    #[clap(short = 'a', long = "address", default_value = "127.0.0.1:5037")]
    pub address: SocketAddrV4,
    /// Serial id of a specific device. Every request will be sent to this device.
    #[clap(short = 's', long = "serial")]
    pub serial: Option<String>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    #[clap(flatten)]
    Local(LocalCommand),
    #[clap(flatten)]
    Host(HostCommand),
    #[clap(flatten)]
    Emu(EmuCommand),
}
