use clap::Parser;
use std::path::PathBuf;

use crate::models::RebootTypeCommand;

#[derive(Parser, Debug)]
pub enum LocalCommand {
    /// List available server features.
    HostFeatures,
    /// Push a file on device
    Push { filename: String, path: String },
    /// Pull a file from device
    Pull { path: String, filename: String },
    /// List a directory on device
    List { path: String },
    /// Stat a file specified on device
    Stat { path: String },
    /// Spawn an interactive shell or run a list of commands on the device
    Shell { commands: Vec<String> },
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
    /// Dump framebuffer of device
    Framebuffer { path: String },
    /// Get logs of device
    Logcat {
        /// Path to output file (created if not exists)
        path: Option<String>,
    },
    /// Install an APK on device
    Install {
        /// Path to APK file. Extension must be ".apk"
        path: PathBuf,
    },
}
