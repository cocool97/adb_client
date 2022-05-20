use std::net::Ipv4Addr;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Args {
    /// Sets the listening address of ADB server
    #[clap(short = 'a', long = "address", default_value = "127.0.0.1")]
    pub address: Ipv4Addr,
    /// Sets the listening port of ADB server
    #[clap(short = 'p', long = "port", default_value = "5037")]
    pub port: u16,
    /// Serial id of a specific device. Every request will be sent to this device.
    #[clap(short = 's', long = "serial")]
    pub serial: Option<String>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Prints current ADB version.
    Version,
    /// Asks ADB server to quit immediately.
    Kill,
    /// List connected devices.
    Devices {
        #[clap(short = 'l', long = "long")]
        long: bool,
    },
    /// Tracks new devices showing up.
    TrackDevices,
    /// Lists available server features.
    HostFeatures,
    /// Run 'command' in a shell on the device, and return its output and error streams.
    Shell { command: Vec<String> },
}
