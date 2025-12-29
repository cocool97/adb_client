use std::path::PathBuf;

use clap::Parser;

use super::RebootTypeCommand;

#[derive(Parser, Debug)]
pub enum DeviceCommands {
    /// Spawn an interactive shell or run a list of commands on the device
    Shell {
        #[arg(trailing_var_arg = true)]
        commands: Vec<String>,
    },
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
        /// The activity to be invoked itself, Usually it is `MainActivity`
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
    /// Uninstall a package from the device
    Uninstall {
        /// Name of the package to uninstall
        package: String,
    },
    /// Dump framebuffer of device
    Framebuffer {
        /// Framebuffer image destination path
        path: String,
    },
    /// List files on device
    List {
        /// Path to list files from
        path: String,
    },
}
