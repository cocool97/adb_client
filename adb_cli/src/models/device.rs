use std::path::PathBuf;

use adb_client::ADBProtoPort;
use clap::Parser;

use super::RebootTypeCommand;

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
    /// Reverse socket connection from remote port to local port
    Reverse {
        /// Remote port
        remote: ADBProtoPort,
        /// Local port
        local: ADBProtoPort,
    },
    /// Remove all previously applied reverse rules
    ReverseRemoveAll,
}
