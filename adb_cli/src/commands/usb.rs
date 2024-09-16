use clap::Parser;

fn hex_id_to_u16(id: &str) -> Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(id, 16)
}

#[derive(Parser, Debug)]
pub struct UsbCommand {
    /// Vendor id of this USB device
    #[clap(short = 'v', long = "vendor-id", value_parser = hex_id_to_u16)]
    pub vendor_id: u16,
    /// Product id of this USB device
    #[clap(short = 'p', long = "product-id", value_parser = hex_id_to_u16)]
    pub product_id: u16,
    // #[clap(subcommand)]
    // pub commands: UsbCommands
}

#[derive(Parser, Debug)]
pub enum UsbCommands {}
