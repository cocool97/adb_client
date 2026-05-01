use std::{fs::File, io::Write};

use adb_client::server_device::ADBServerDevice;

use crate::models::{ADBCliResult, ForwardCommand, LocalDeviceCommand, ReverseCommand};

pub fn handle_local_commands(
    mut device: ADBServerDevice,
    local_device_commands: LocalDeviceCommand,
) -> ADBCliResult<()> {
    match local_device_commands {
        LocalDeviceCommand::HostFeatures => {
            let features = device
                .host_features()?
                .iter()
                .map(ToString::to_string)
                .reduce(|a, b| format!("{a},{b}"))
                .unwrap_or_default();
            log::info!("Available host features: {features}");

            Ok(())
        }
        LocalDeviceCommand::Logcat { path } => {
            let writer: Box<dyn Write> = if let Some(path) = path {
                let log_file = File::create(path)?;
                Box::new(log_file)
            } else {
                Box::new(std::io::stdout())
            };
            Ok(device.get_logs(writer)?)
        }
        LocalDeviceCommand::Forward(forward_command) => match forward_command {
            ForwardCommand::RemoveAll => Ok(device.forward_remove_all()?),
            ForwardCommand::Remove { local } => Ok(device.forward_remove(local)?),
            ForwardCommand::Add { local, remote } => Ok(device.forward(local, remote)?),
        },
        LocalDeviceCommand::Reverse(reverse_command) => match reverse_command {
            ReverseCommand::RemoveAll => Ok(device.reverse_remove_all()?),
            ReverseCommand::Remove { local } => Ok(device.reverse_remove(local)?),
            ReverseCommand::Add { remote, local } => Ok(device.reverse(remote, local)?),
        },
    }
}
