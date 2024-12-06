use std::net::SocketAddrV4;

use clap::{Parser, Subcommand};

use super::{EmulatorCommand, HostCommand, LocalCommand, TcpCommand, UsbCommand};

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Opts {
    #[clap(long = "debug")]
    pub debug: bool,
    #[clap(subcommand)]
    pub command: MainCommand,
}

#[derive(Debug, Parser)]
pub enum MainCommand {
    /// Server related commands
    Host(ServerCommand<HostCommand>),
    /// Device related commands using server
    Local(ServerCommand<LocalCommand>),
    /// Emulator related commands
    Emu(EmulatorCommand),
    /// USB device related commands
    Usb(UsbCommand),
    /// TCP device related commands
    Tcp(TcpCommand),
    /// MDNS discovery related commands
    Mdns,
}

#[derive(Debug, Parser)]
pub struct ServerCommand<T: Subcommand> {
    #[clap(short = 'a', long = "address", default_value = "127.0.0.1:5037")]
    pub address: SocketAddrV4,
    /// Serial id of a specific device. Every request will be sent to this device.
    #[clap(short = 's', long = "serial")]
    pub serial: Option<String>,
    #[clap(subcommand)]
    pub command: T,
}
