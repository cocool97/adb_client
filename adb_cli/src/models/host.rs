use std::{net::SocketAddrV4, str::FromStr};

use adb_client::{RustADBError, WaitForDeviceTransport};
use clap::Parser;

fn parse_wait_for_device_device_transport(
    value: &str,
) -> Result<WaitForDeviceTransport, RustADBError> {
    WaitForDeviceTransport::from_str(value)
}

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
    /// MDNS services
    Mdns {
        #[clap(subcommand)]
        subcommand: MdnsCommand,
    },
    /// Display server status
    ServerStatus,
    /// Wait for a device, on optionally given transport
    WaitForDevice {
        /// Transport on which wait for devices
        #[clap(short = 't', long = "transport", value_parser = parse_wait_for_device_device_transport)]
        transport: Option<WaitForDeviceTransport>,
    },
}

#[derive(Parser, Debug)]
pub enum MdnsCommand {
    /// Check mdns status
    Check,
    /// List mdns services available
    Services,
}
