#![doc = include_str!("../README.md")]

mod commands;
mod models;

use adb_client::{ADBEmulatorDevice, ADBServer, ADBUSBDevice, DeviceShort};
use anyhow::{anyhow, Result};
use clap::Parser;
use commands::{EmuCommand, HostCommand, LocalCommand};
use env_logger::Builder;
use log::LevelFilter;
use models::{Command, Opts};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> Result<()> {
    let opt = Opts::parse();

    let max_level = if opt.verbose {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };
    let mut builder = Builder::default();
    builder.filter_level(max_level);
    builder.init();

    match opt.command {
        Command::Local(local) => {
            let mut adb_server = ADBServer::new(opt.address);

            let mut device = match opt.serial {
                Some(serial) => adb_server.get_device_by_name(&serial)?,
                None => adb_server.get_device()?,
            };

            match local {
                LocalCommand::Pull { path, filename } => {
                    let mut output = File::create(Path::new(&filename))?;
                    device.recv(&path, &mut output)?;
                    log::info!("Downloaded {path} as {filename}");
                }
                LocalCommand::Push { filename, path } => {
                    let mut input = File::open(Path::new(&filename))?;
                    device.send(&mut input, &path)?;
                    log::info!("Uploaded {filename} to {path}");
                }
                LocalCommand::List { path } => {
                    device.list(path)?;
                }
                LocalCommand::Stat { path } => {
                    let stat_response = device.stat(path)?;
                    log::info!("{}", stat_response);
                }
                LocalCommand::Shell { command } => {
                    if command.is_empty() {
                        device.shell()?;
                    } else {
                        device.shell_command(command, std::io::stdout())?;
                    }
                }
                LocalCommand::HostFeatures => {
                    let features = device
                        .host_features()?
                        .iter()
                        .map(|v| v.to_string())
                        .reduce(|a, b| format!("{a},{b}"))
                        .ok_or(anyhow!("cannot list features"))?;
                    log::info!("Available host features: {features}");
                }
                LocalCommand::Reboot { sub_command } => {
                    log::info!("Reboots device");
                    device.reboot(sub_command.into())?
                }
                LocalCommand::Framebuffer { path } => {
                    device.framebuffer(&path)?;
                    log::info!("Framebuffer dropped: {path}");
                }
                LocalCommand::Logcat { path } => {
                    let writer: Box<dyn Write> = if let Some(path) = path {
                        let f = File::create(path)?;
                        Box::new(f)
                    } else {
                        Box::new(std::io::stdout())
                    };
                    device.get_logs(writer)?;
                }
            }
        }
        Command::Host(host) => {
            let mut adb_server = ADBServer::new(opt.address);

            match host {
                HostCommand::Version => {
                    let version = adb_server.version()?;
                    log::info!("Android Debug Bridge version {}", version);
                    log::info!("Package version {}-rust", std::env!("CARGO_PKG_VERSION"));
                }
                HostCommand::Kill => {
                    adb_server.kill()?;
                }
                HostCommand::Devices { long } => {
                    if long {
                        log::info!("List of devices attached (extended)");
                        for device in adb_server.devices_long()? {
                            log::info!("{}", device);
                        }
                    } else {
                        log::info!("List of devices attached");
                        for device in adb_server.devices()? {
                            log::info!("{}", device);
                        }
                    }
                }
                HostCommand::TrackDevices => {
                    let callback = |device: DeviceShort| {
                        log::info!("{}", device);
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
            }
        }
        Command::Emu(emu) => {
            let mut emulator = match opt.serial {
                Some(serial) => ADBEmulatorDevice::new(serial, None)?,
                None => return Err(anyhow!("Serial must be set to use emulators !")),
            };

            match emu {
                EmuCommand::Sms {
                    phone_number,
                    content,
                } => {
                    emulator.send_sms(&phone_number, &content)?;
                    log::info!("SMS sent to {phone_number}");
                }
                EmuCommand::Rotate => emulator.rotate()?,
            }
        }
        Command::Usb(usb) => {
            let mut device =
                ADBUSBDevice::new(usb.vendor_id, usb.product_id, usb.path_to_private_key)?;
            device.send_connect()?;
        }
    }

    Ok(())
}
