use std::{fs::File, io::Write};

use adb_client::{ADBListItemType, ADBServerDevice};
use anyhow::{anyhow, Result};

use crate::models::LocalDeviceCommand;

pub fn handle_local_commands(
    mut device: ADBServerDevice,
    local_device_commands: LocalDeviceCommand,
) -> Result<()> {
    match local_device_commands {
        LocalDeviceCommand::HostFeatures => {
            let features = device
                .host_features()?
                .iter()
                .map(|v| v.to_string())
                .reduce(|a, b| format!("{a},{b}"))
                .ok_or(anyhow!("cannot list features"))?;
            log::info!("Available host features: {features}");

            Ok(())
        }
        LocalDeviceCommand::List { path } => {
            let dirs = device.list(path)?;
            for dir in dirs {
                let list_item_type = match dir.item_type {
                    ADBListItemType::File => "File".to_string(),
                    ADBListItemType::Directory => "Dir".to_string(),
                    ADBListItemType::Symlink => "Symlink".to_string(),
                };
                log::info!(
                    "type: {}, name: {}, time: {}, size: {}, permissions: {:#o}",
                    list_item_type,
                    dir.name,
                    dir.time,
                    dir.size,
                    dir.permissions
                );
            }
            Ok(())
        }
        LocalDeviceCommand::Logcat { path } => {
            let writer: Box<dyn Write> = if let Some(path) = path {
                let f = File::create(path)?;
                Box::new(f)
            } else {
                Box::new(std::io::stdout())
            };
            Ok(device.get_logs(writer)?)
        }
    }
}
