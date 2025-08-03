use adb_client::{ADBServer, DeviceShort, MDNSBackend, Result, WaitForDeviceState};

use crate::models::{HostCommand, MdnsCommand, ServerCommand};

pub fn handle_host_commands(server_command: ServerCommand<HostCommand>) -> Result<()> {
    let mut adb_server = ADBServer::new(server_command.address);

    match server_command.command {
        HostCommand::Version => {
            let version = adb_server.version()?;
            log::info!("Android Debug Bridge version {version}");
            log::info!("Package version {}-rust", std::env!("CARGO_PKG_VERSION"));
        }
        HostCommand::Kill => {
            adb_server.kill()?;
        }
        HostCommand::Devices { long } => {
            if long {
                log::info!("List of devices attached (extended)");
                for device in adb_server.devices_long()? {
                    log::info!("{device}");
                }
            } else {
                log::info!("List of devices attached");
                for device in adb_server.devices()? {
                    log::info!("{device}");
                }
            }
        }
        HostCommand::TrackDevices => {
            let callback = |device: DeviceShort| {
                log::info!("{device}");
                Ok(())
            };
            log::info!("Live list of devices attached");
            adb_server.track_devices(callback)?;
        }
        HostCommand::Pair { address, code } => {
            adb_server.pair(address, code)?;
            log::info!("Paired device {address}");
        }
        HostCommand::Connect { address } => {
            adb_server.connect_device(address)?;
            log::info!("Connected to {address}");
        }
        HostCommand::Disconnect { address } => {
            adb_server.disconnect_device(address)?;
            log::info!("Disconnected {address}");
        }
        HostCommand::Mdns { subcommand } => match subcommand {
            MdnsCommand::Check => {
                let check = adb_server.mdns_check()?;
                let server_status = adb_server.server_status()?;
                match server_status.mdns_backend {
                    MDNSBackend::Unknown => log::info!("unknown mdns backend..."),
                    MDNSBackend::Bonjour => match check {
                        true => log::info!("mdns daemon version [Bonjour]"),
                        false => log::info!("ERROR: mdns daemon unavailable"),
                    },
                    MDNSBackend::OpenScreen => {
                        log::info!("mdns daemon version [Openscreen discovery 0.0.0]")
                    }
                }
            }
            MdnsCommand::Services => {
                log::info!("List of discovered mdns services");
                for service in adb_server.mdns_services()? {
                    log::info!("{service}");
                }
            }
        },
        HostCommand::ServerStatus => {
            log::info!("{}", adb_server.server_status()?);
        }
        HostCommand::WaitForDevice { transport } => {
            log::info!("waiting for device to be connected...");
            adb_server.wait_for_device(WaitForDeviceState::Device, transport)?;
        }
    }

    Ok(())
}
