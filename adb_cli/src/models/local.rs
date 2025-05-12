use clap::Parser;

use super::DeviceCommands;

#[derive(Parser, Debug)]
pub enum LocalCommand {
    #[clap(flatten)]
    DeviceCommands(DeviceCommands),
    #[clap(flatten)]
    LocalDeviceCommand(LocalDeviceCommand),
}

#[derive(Parser, Debug)]
pub enum LocalDeviceCommand {
    /// List available server features.
    HostFeatures,
    /// Get logs of device
    Logcat {
        /// Path to output file (created if not exists)
        path: Option<String>,
    },
}
