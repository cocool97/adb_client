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
}

#[derive(Parser, Debug)]
pub enum UsbCommands {

}
