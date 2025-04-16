#![doc = include_str!("../README.md")]

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod adb_termios;

mod handlers;
mod models;
mod utils;

use adb_client::{
    ADBDeviceExt, ADBServer, ADBServerDevice, ADBTcpDevice, ADBUSBDevice, MDNSDiscoveryService,
};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use adb_termios::ADBTermios;

use anyhow::Result;
use clap::Parser;
use handlers::{handle_emulator_commands, handle_host_commands, handle_local_commands};
use models::{DeviceCommands, LocalCommand, MainCommand, Opts};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use utils::setup_logger;

fn main() -> Result<()> {
    // This depends on `clap`
    let opts = Opts::parse();

    // SAFETY:
    // We are assuming the entire process is single-threaded
    // at this point.
    // This seems true for the current version of `clap`,
    // but there's no guarantee for future updates
    unsafe { setup_logger(opts.debug) };

    // Directly handling methods / commands that aren't linked to [`ADBDeviceExt`] trait.
    // Other methods just have to create a concrete [`ADBDeviceExt`] instance, and return it.
    // This instance will then be used to execute desired command.
    let (mut device, commands) = match opts.command {
        MainCommand::Host(server_command) => return Ok(handle_host_commands(server_command)?),
        MainCommand::Emu(emulator_command) => return handle_emulator_commands(emulator_command),
        MainCommand::Local(server_command) => {
            // Must start server to communicate with device, but only if this is a local one.
            let server_address_ip = server_command.address.ip();
            if server_address_ip.is_loopback() || server_address_ip.is_unspecified() {
                ADBServer::start(&HashMap::default(), &None);
            }

            let device = match server_command.serial {
                Some(serial) => ADBServerDevice::new(serial, Some(server_command.address)),
                None => ADBServerDevice::autodetect(Some(server_command.address)),
            };

            match server_command.command {
                LocalCommand::DeviceCommands(device_commands) => (device.boxed(), device_commands),
                LocalCommand::LocalDeviceCommand(local_device_command) => {
                    return handle_local_commands(device, local_device_command);
                }
            }
        }
        MainCommand::Usb(usb_command) => {
            let device = match (usb_command.vendor_id, usb_command.product_id) {
                (Some(vid), Some(pid)) => match usb_command.path_to_private_key {
                    Some(pk) => ADBUSBDevice::new_with_custom_private_key(vid, pid, pk)?,
                    None => ADBUSBDevice::new(vid, pid)?,
                },
                (None, None) => match usb_command.path_to_private_key {
                    Some(pk) => ADBUSBDevice::autodetect_with_custom_private_key(pk)?,
                    None => ADBUSBDevice::autodetect()?,
                },
                _ => {
                    anyhow::bail!(
                        "please either supply values for both the --vendor-id and --product-id flags or none."
                    );
                }
            };
            (device.boxed(), usb_command.commands)
        }
        MainCommand::Tcp(tcp_command) => {
            let device = ADBTcpDevice::new(tcp_command.address)?;
            (device.boxed(), tcp_command.commands)
        }
        MainCommand::Mdns => {
            let mut service = MDNSDiscoveryService::new()?;

            let (tx, rx) = std::sync::mpsc::channel();
            service.start(tx)?;

            log::info!("Starting mdns discovery...");
            while let Ok(device) = rx.recv() {
                log::info!(
                    "Found device {} with addresses {:?}",
                    device.fullname,
                    device.addresses
                )
            }

            return Ok(service.shutdown()?);
        }
    };

    match commands {
        DeviceCommands::Shell { commands } => {
            if commands.is_empty() {
                // Need to duplicate some code here as ADBTermios [Drop] implementation resets terminal state.
                // Using a scope here would call drop() too early..
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                {
                    let mut adb_termios = ADBTermios::new(std::io::stdin())?;
                    adb_termios.set_adb_termios()?;
                    device.shell(&mut std::io::stdin(), Box::new(std::io::stdout()))?;
                }

                #[cfg(not(any(target_os = "linux", target_os = "macos")))]
                {
                    device.shell(&mut std::io::stdin(), Box::new(std::io::stdout()))?;
                }
            } else {
                let commands: Vec<&str> = commands.iter().map(|v| v.as_str()).collect();
                device.shell_command(&commands, &mut std::io::stdout())?;
            }
        }
        DeviceCommands::Pull {
            source,
            destination,
        } => {
            let mut output = File::create(Path::new(&destination))?;
            device.pull(&source, &mut output)?;
            log::info!("Downloaded {source} as {destination}");
        }
        DeviceCommands::Stat { path } => {
            let stat_response = device.stat(&path)?;
            println!("{}", stat_response);
        }
        DeviceCommands::Reboot { reboot_type } => {
            log::info!("Reboots device in mode {:?}", reboot_type);
            device.reboot(reboot_type.into())?
        }
        DeviceCommands::Push { filename, path } => {
            let mut input = File::open(Path::new(&filename))?;
            device.push(&mut input, &path)?;
            log::info!("Uploaded {filename} to {path}");
        }
        DeviceCommands::Run { package, activity } => {
            let output = device.run_activity(&package, &activity)?;
            std::io::stdout().write_all(&output)?;
        }
        DeviceCommands::Install { path } => {
            log::info!("Starting installation of APK {}...", path.display());
            device.install(&path)?;
        }
        DeviceCommands::Uninstall { package } => {
            log::info!("Uninstalling the package {}...", package);
            device.uninstall(&package)?;
        }
        DeviceCommands::Framebuffer { path } => {
            device.framebuffer(&path)?;
            log::info!("Successfully dumped framebuffer at path {path}");
        }
    }

    Ok(())
}
