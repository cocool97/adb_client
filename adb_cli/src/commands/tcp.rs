use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;

use crate::models::RebootTypeCommand;

#[derive(Parser, Debug)]
pub struct TcpCommand {
    pub address: SocketAddr,
    #[clap(subcommand)]
    pub commands: TcpCommands,
}

#[derive(Parser, Debug)]
pub enum TcpCommands {
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
}
