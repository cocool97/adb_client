use std::num::ParseIntError;
use std::path::PathBuf;

use clap::Parser;

use crate::models::RebootTypeCommand;

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
    #[clap(subcommand)]
    pub commands: DeviceCommands,
}

#[derive(Parser, Debug)]
pub enum DeviceCommands {
    /// Spawn an interactive shell or run a list of commands on the device
    Shell { commands: Vec<String> },
    /// Pull a file from device
    Pull { source: String, destination: String },
    /// Push a file on device
    Push { filename: String, path: String },
    /// Stat a file on device
    Stat { path: String },
    /// Run an activity on device specified by the intent
    Run {
        /// The package whose activity is to be invoked
        #[clap(short = 'p', long = "package")]
        package: String,
        /// The activity to be invoked itself, Usually it is MainActivity
        #[clap(short = 'a', long = "activity")]
        activity: String,
    },
    /// Reboot the device
    Reboot {
        #[clap(subcommand)]
        reboot_type: RebootTypeCommand,
    },
    /// Install an APK on device
    Install {
        /// Path to APK file. Extension must be ".apk"
        path: PathBuf,
    },
    /// Dump framebuffer of device
    Framebuffer {
        /// Framebuffer image destination path
        path: String,
    },
}
