use std::net::SocketAddrV4;

use clap::Parser;

#[derive(Parser, Debug)]
pub enum HostCommand {
    /// Print current ADB version.
    Version,
    /// Ask ADB server to quit immediately.
    Kill,
    /// List connected devices.
    Devices {
        #[clap(short = 'l', long = "long")]
        long: bool,
    },
    /// Track new devices showing up.
    TrackDevices,
    /// Pair device with a given code
    Pair { address: SocketAddrV4, code: String },
    /// Connect device over WI-FI
    Connect { address: SocketAddrV4 },
    /// Disconnect device over WI-FI
    Disconnect { address: SocketAddrV4 },
}
