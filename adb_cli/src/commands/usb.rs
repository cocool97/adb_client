use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct UsbCommand {
    /// Vendor id of this USB device
    #[clap(short = 'v', long = "vendor-id")]
    pub vendor_id: u16,
    /// Product id of this USB device
    #[clap(short = 'p', long = "product-id")]
    pub product_id: u16,
    // #[clap(subcommand)]
    // pub commands: UsbCommands
    /// Path to a custom public key for authentication
    #[clap(long = "public-key")]
    pub public_key: Option<PathBuf>,

    /// Path to a custom private key for authentication
    #[clap(long = "private-key")]
    pub private_key: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub enum UsbCommands {}
