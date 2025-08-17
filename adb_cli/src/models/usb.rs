use std::num::ParseIntError;
use std::path::PathBuf;

use clap::Parser;

use super::DeviceCommands;

fn parse_hex_id(id: &str) -> Result<u16, ParseIntError> {
    u16::from_str_radix(id, 16)
}

#[derive(Parser, Debug)]
pub struct UsbCommand {
    /// Hexadecimal vendor id of this USB device
    #[clap(short = 'v', long = "vendor-id", value_parser=parse_hex_id, value_name="VID")]
    pub vendor_id: Option<u16>,
    /// Hexadecimal product id of this USB device
    #[clap(short = 'p', long = "product-id", value_parser=parse_hex_id, value_name="PID")]
    pub product_id: Option<u16>,
    /// Path to a custom private key to use for authentication
    #[clap(short = 'k', long = "private-key")]
    pub path_to_private_key: Option<PathBuf>,
    /// List all connected Android devices
    #[clap(short = 'l', long = "list")]
    pub list_devices: bool,
    #[clap(subcommand)]
    pub commands: Option<DeviceCommands>,
}
