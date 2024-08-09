mod commands;
mod models;

use adb_client::{ADBEmulatorDevice, ADBServer, DeviceShort};
use anyhow::{anyhow, Result};
use clap::Parser;
use commands::{EmuCommand, HostCommand, LocalCommand};
use models::{Command, Opts};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

fn main() -> Result<()> {
    let opt = Opts::parse();

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
                    println!("Downloaded {path} as {filename}");
                }
                LocalCommand::Push { filename, path } => {
                    let mut input = File::open(Path::new(&filename))?;
                    device.send(&mut input, &path)?;
                    println!("Uploaded {filename} to {path}");
                }
                LocalCommand::List { path } => {
                    device.list(path)?;
                }
                LocalCommand::Stat { path } => {
                    let stat_response = device.stat(path)?;
                    println!("{}", stat_response);
                }
                LocalCommand::Shell { command } => {
                    if command.is_empty() {
                        device.shell()?;
                    } else {
                        let stdout = device.shell_command(command)?;
                        io::stdout().write_all(&stdout)?;
                    }
                }
                LocalCommand::HostFeatures => {
                    println!("Available host features");
                    for feature in device.host_features()? {
                        println!("- {}", feature);
                    }
                }
                LocalCommand::Reboot { sub_command } => {
                    println!("Reboots device");
                    device.reboot(sub_command.into())?
                }
                LocalCommand::Framebuffer { path } => {
                    device.framebuffer(&path)?;
                    println!("Framebuffer dropped: {path}");
                }
            }
        }
        Command::Host(host) => {
            let mut adb_server = ADBServer::new(opt.address);

            match host {
                HostCommand::Version => {
                    let version = adb_server.version()?;
                    println!("Android Debug Bridge version {}", version);
                    println!("Package version {}-rust", std::env!("CARGO_PKG_VERSION"));
                }
                HostCommand::Kill => {
                    adb_server.kill()?;
                }
                HostCommand::Devices { long } => {
                    if long {
                        println!("List of devices attached (extended)");
                        for device in adb_server.devices_long()? {
                            println!("{}", device);
                        }
                    } else {
                        println!("List of devices attached");
                        for device in adb_server.devices()? {
                            println!("{}", device);
                        }
                    }
                }
                HostCommand::TrackDevices => {
                    let callback = |device: DeviceShort| {
                        println!("{}", device);
                        Ok(())
                    };
                    println!("Live list of devices attached");
                    adb_server.track_devices(callback)?;
                }
                HostCommand::Pair { address, code } => {
                    adb_server.pair(address, code)?;
                    println!("paired device {address}");
                }
                HostCommand::Connect { address } => {
                    adb_server.connect_device(address)?;
                    println!("connected to {address}");
                }
                HostCommand::Disconnect { address } => {
                    adb_server.disconnect_device(address)?;
                    println!("disconnected {address}");
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
                    println!("sms sent...");
                }
                EmuCommand::Rotate => emulator.rotate()?,
            }
        }
    }

    Ok(())
}
