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
    /// Stat a file on device with extended information
    StatExtended { path: String },
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
        /// User id to install the package for
        #[clap(short = 'u', long = "user")]
        user: Option<String>,
        /// Path to APK file. Extension must be ".apk"
        path: PathBuf,
    },
    /// Uninstall a package from the device
    Uninstall {
        /// User id of the package to uninstall
        #[clap(short = 'u', long = "user")]
        user: Option<String>,
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
    /// Restart adb daemon with root permissions
    Root,
    /// Forward a host endpoint to a device endpoint.
    #[clap(subcommand)]
    Forward(ForwardCommand),
    /// Reverse forward a device endpoint to a host endpoint.
    #[clap(subcommand)]
    Reverse(ReverseCommand),
}

/// Over USB or TCP (without ADB server), connections are relayed by this process: `add` keeps forwarding until
/// interrupted (Ctrl-C), then removes the forward.
#[derive(Parser, Debug)]
pub enum ForwardCommand {
    /// Remove all forwards.
    RemoveAll,
    /// Remove the forward of a host endpoint.
    Remove {
        /// Host endpoint, e.g. tcp:8080
        local: String,
    },
    /// Forward a host endpoint to a device endpoint. Over USB or TCP, blocks until interrupted (Ctrl-C).
    Add {
        /// Host endpoint, e.g. tcp:8080 (tcp:0 to allocate a port)
        local: String,
        /// Device endpoint, e.g. tcp:8080 or localabstract:name
        remote: String,
    },
}

/// Over USB or TCP (without ADB server), connections are relayed by this process: `add` keeps forwarding until
/// interrupted (Ctrl-C), then removes the reverse forward.
#[derive(Parser, Debug)]
pub enum ReverseCommand {
    /// Remove all reverse forwards.
    RemoveAll,
    /// Remove the reverse forward of a device endpoint.
    Remove {
        /// Device endpoint, e.g. tcp:8080
        remote: String,
    },
    /// Reverse forward a device endpoint to a host endpoint. Over USB or TCP, blocks until interrupted (Ctrl-C).
    Add {
        /// Device endpoint, e.g. tcp:8080
        remote: String,
        /// Host endpoint, e.g. tcp:8080
        local: String,
    },
}
