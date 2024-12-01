#![doc = include_str!("../README.md")]

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod adb_termios;

mod commands;
mod models;

use adb_client::{
    ADBDeviceExt, ADBEmulatorDevice, ADBServer, ADBTcpDevice, ADBUSBDevice, DeviceShort,
    MDNSBackend, MDNSDiscoveryService,
};
use anyhow::{anyhow, Result};
use clap::Parser;
use commands::{EmuCommand, HostCommand, LocalCommand, TcpCommands, UsbCommands};
use models::{Command, Opts};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> Result<()> {
    let opt = Opts::parse();

    // Setting default log level as "info" if not set
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

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
                    device.pull(&path, &mut output)?;
                    log::info!("Downloaded {path} as {filename}");
                }
                LocalCommand::Push { filename, path } => {
                    let mut input = File::open(Path::new(&filename))?;
                    device.push(&mut input, &path)?;
                    log::info!("Uploaded {filename} to {path}");
                }
                LocalCommand::List { path } => {
                    device.list(path)?;
                }
                LocalCommand::Stat { path } => {
                    let stat_response = device.stat(path)?;
                    println!("{}", stat_response);
                }
                LocalCommand::Shell { commands } => {
                    if commands.is_empty() {
                        // Need to duplicate some code here as ADBTermios [Drop] implementation resets terminal state.
                        // Using a scope here would call drop() too early..
                        #[cfg(any(target_os = "linux", target_os = "macos"))]
                        {
                            let mut adb_termios = adb_termios::ADBTermios::new(std::io::stdin())?;
                            adb_termios.set_adb_termios()?;
                            device.shell(std::io::stdin(), std::io::stdout())?;
                        }

                        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
                        {
                            device.shell(std::io::stdin(), std::io::stdout())?;
                        }
                    } else {
                        device.shell_command(commands, std::io::stdout())?;
                    }
                }
                LocalCommand::Run { package, activity } => {
                    let output = device.run_activity(&package, &activity)?;
                    std::io::stdout().write_all(&output)?;
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
                LocalCommand::Reboot { reboot_type } => {
                    log::info!("Reboots device in mode {:?}", reboot_type);
                    device.reboot(reboot_type.into())?
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
                LocalCommand::Install { path } => {
                    log::info!("Starting installation of APK {}...", path.display());
                    device.install(path)?;
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
                HostCommand::Mdns { command } => {
                    if command == "check" {
                        let check = adb_server.mdns_check()?;
                        let server_status = adb_server.server_status()?;
                        if server_status.mdns_backend == MDNSBackend::Bonjour {
                            if check {
                                log::info!("mdns daemon version [Bonjour]");
                            } else {
                                log::info!("ERROR: mdns daemon unavailable");
                            }
                        } else {
                            log::info!("mdns daemon version [Openscreen discovery 0.0.0]");
                        }
                    } else if command == "services" {
                        log::info!("List of discovered mdns services");
                        for service in adb_server.mdns_services()? {
                            log::info!("{}", service);
                        }
                    }
                }
                HostCommand::ServerStatus => {
                    log::info!("{}", adb_server.server_status()?);
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
            let mut device = match (usb.vendor_id, usb.product_id) {
                (Some(vid), Some(pid)) => match usb.path_to_private_key {
                    Some(pk) => ADBUSBDevice::new_with_custom_private_key(vid, pid, pk)?,
                    None => ADBUSBDevice::new(vid, pid)?,
                },

                (None, None) => match usb.path_to_private_key {
                    Some(pk) => ADBUSBDevice::autodetect_with_custom_private_key(pk)?,
                    None => ADBUSBDevice::autodetect()?,
                },

                _ => {
                    anyhow::bail!("please either supply values for both the --vendor-id and --product-id flags or none.");
                }
            };

            match usb.commands {
                UsbCommands::Shell { commands } => {
                    if commands.is_empty() {
                        // Need to duplicate some code here as ADBTermios [Drop] implementation resets terminal state.
                        // Using a scope here would call drop() too early..
                        #[cfg(any(target_os = "linux", target_os = "macos"))]
                        {
                            let mut adb_termios = adb_termios::ADBTermios::new(std::io::stdin())?;
                            adb_termios.set_adb_termios()?;
                            device.shell(std::io::stdin(), std::io::stdout())?;
                        }

                        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
                        {
                            device.shell(std::io::stdin(), std::io::stdout())?;
                        }
                    } else {
                        device.shell_command(commands, std::io::stdout())?;
                    }
                }
                UsbCommands::Pull {
                    source,
                    destination,
                } => {
                    let mut output = File::create(Path::new(&destination))?;
                    device.pull(&source, &mut output)?;
                    log::info!("Downloaded {source} as {destination}");
                }
                UsbCommands::Stat { path } => {
                    let stat_response = device.stat(&path)?;
                    println!("{}", stat_response);
                }
                UsbCommands::Reboot { reboot_type } => {
                    log::info!("Reboots device in mode {:?}", reboot_type);
                    device.reboot(reboot_type.into())?
                }
                UsbCommands::Push { filename, path } => {
                    let mut input = File::open(Path::new(&filename))?;
                    device.push(&mut input, &path)?;
                    log::info!("Uploaded {filename} to {path}");
                }
                UsbCommands::Run { package, activity } => {
                    let output = device.run_activity(&package, &activity)?;
                    std::io::stdout().write_all(&output)?;
                }
                UsbCommands::Install { path } => {
                    log::info!("Starting installation of APK {}...", path.display());
                    device.install(path)?;
                }
            }
        }
        Command::Tcp(tcp) => {
            let mut device = ADBTcpDevice::new(tcp.address)?;

            match tcp.commands {
                TcpCommands::Shell { commands } => {
                    if commands.is_empty() {
                        // Need to duplicate some code here as ADBTermios [Drop] implementation resets terminal state.
                        // Using a scope here would call drop() too early..
                        #[cfg(any(target_os = "linux", target_os = "macos"))]
                        {
                            let mut adb_termios = adb_termios::ADBTermios::new(std::io::stdin())?;
                            adb_termios.set_adb_termios()?;
                            device.shell(std::io::stdin(), std::io::stdout())?;
                        }

                        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
                        {
                            device.shell(std::io::stdin(), std::io::stdout())?;
                        }
                    } else {
                        device.shell_command(commands, std::io::stdout())?;
                    }
                }
                TcpCommands::Pull {
                    source,
                    destination,
                } => {
                    let mut output = File::create(Path::new(&destination))?;
                    device.pull(&source, &mut output)?;
                    log::info!("Downloaded {source} as {destination}");
                }
                TcpCommands::Stat { path } => {
                    let stat_response = device.stat(&path)?;
                    println!("{}", stat_response);
                }
                TcpCommands::Reboot { reboot_type } => {
                    log::info!("Reboots device in mode {:?}", reboot_type);
                    device.reboot(reboot_type.into())?
                }
                TcpCommands::Push { filename, path } => {
                    let mut input = File::open(Path::new(&filename))?;
                    device.push(&mut input, &path)?;
                    log::info!("Uploaded {filename} to {path}");
                }
                TcpCommands::Run { package, activity } => {
                    let output = device.run_activity(&package, &activity)?;
                    std::io::stdout().write_all(&output)?;
                }
                TcpCommands::Install { path } => {
                    log::info!("Starting installation of APK {}...", path.display());
                    device.install(path)?;
                }
            }
        }
        Command::Mdns => {
            let mut service = MDNSDiscoveryService::new()?;

            let (tx, rx) = std::sync::mpsc::channel();
            service.start(tx)?;

            while let Ok(device) = rx.recv() {
                log::info!(
                    "Found device {} with addresses {:?}",
                    device.fullname,
                    device.addresses
                )
            }

            service.shutdown()?;
        }
    }

    Ok(())
}
