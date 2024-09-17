use std::num::ParseIntError;

use clap::Parser;

fn parse_hex_id(id: &str) -> Result<u16, ParseIntError> {
    u16::from_str_radix(id, 16)
}

#[derive(Parser, Debug)]
pub struct UsbCommand {
    /// Vendor id of this USB device
    #[clap(short = 'v', long = "vendor-id", value_parser=parse_hex_id)]
    pub vendor_id: u16,
    /// Product id of this USB device
    #[clap(short = 'p', long = "product-id", value_parser=parse_hex_id)]
    pub product_id: u16,
    // #[clap(subcommand)]
    // pub commands: UsbCommands
}

#[derive(Parser, Debug)]
pub enum UsbCommands {}
