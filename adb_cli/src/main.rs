#![doc = include_str!("../README.md")]

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod adb_termios;

mod handlers;
mod models;
mod utils;

use adb_client::ADBDeviceExt;
use adb_client::mdns::MDNSDiscoveryService;
use adb_client::server::ADBServer;
use adb_client::server_device::ADBServerDevice;
use adb_client::tcp::ADBTcpDevice;
use adb_client::usb::{ADBDeviceInfo, ADBUSBDevice, find_all_connected_adb_devices};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use adb_termios::ADBTermios;

use clap::Parser;
use handlers::{handle_emulator_commands, handle_host_commands, handle_local_commands};
use models::{DeviceCommands, LocalCommand, MainCommand, Opts};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, stdout};
use std::path::Path;
use std::process::ExitCode;
use tabwriter::TabWriter;
use utils::setup_logger;

use crate::models::{ADBCliError, ADBCliResult};

fn run_command(mut device: Box<dyn ADBDeviceExt>, command: DeviceCommands) -> ADBCliResult<()> {
    match command {
        DeviceCommands::Shell { commands } => {
            if commands.is_empty() {
                // Need to duplicate some code here as ADBTermios [Drop] implementation resets terminal state.
                // Using a scope here would call drop() too early..
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                {
                    let mut adb_termios = ADBTermios::new(&std::io::stdin())?;
                    adb_termios.set_adb_termios()?;
                    device.shell(&mut std::io::stdin(), Box::new(std::io::stdout()))?;
                }

                #[cfg(not(any(target_os = "linux", target_os = "macos")))]
                {
                    device.shell(&mut std::io::stdin(), Box::new(std::io::stdout()))?;
                }
            } else {
                device.shell_command(&commands.join(" "), &mut std::io::stdout())?;
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
            println!("{stat_response}");
        }
        DeviceCommands::Reboot { reboot_type } => {
            log::info!("Reboots device in mode {reboot_type:?}");
            device.reboot(reboot_type.into())?;
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
        DeviceCommands::Uninstall { package, user } => {
            log::info!("Uninstalling the package {package}...");
            device.uninstall(&package, user.as_deref())?;
        }
        DeviceCommands::Framebuffer { path } => {
            device.framebuffer(&path)?;
            log::info!("Successfully dumped framebuffer at path {path}");
        }
        DeviceCommands::List { path } => {
            let dirs = device.list(&path)?;
            for dir in dirs {
                log::info!("{dir}");
            }
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    if let Err(err) = inner_main() {
        log::error!("{err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn inner_main() -> ADBCliResult<()> {
    // This depends on `clap`
    let opts = Opts::parse();

    setup_logger(opts.debug);

    // Directly handling methods / commands that aren't linked to [`ADBDeviceExt`] trait.
    // Other methods just have to create a concrete [`ADBDeviceExt`] instance, and return it.
    // This instance will then be used to execute desired command.
    let (device, commands) = match opts.command {
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
            if usb_command.list_devices {
                let devices = find_all_connected_adb_devices()?;

                let mut writer = TabWriter::new(stdout()).alignment(tabwriter::Alignment::Center);
                writeln!(writer, "Index\tVendor ID\tProduct ID\tDevice Description")?;
                writeln!(writer, "-----\t---------\t----------\t----------------")?;

                for (
                    index,
                    ADBDeviceInfo {
                        vendor_id,
                        product_id,
                        device_description,
                    },
                ) in devices.iter().enumerate()
                {
                    writeln!(
                        writer,
                        "#{index}\t{vendor_id:04x}\t{product_id:04x}\t{device_description}",
                    )?;
                }

                writer.flush()?;

                return Ok(());
            }

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
                    return Err(ADBCliError::Standard(
                        "cannot specify flags --vendor-id without --product-id or vice versa"
                            .into(),
                    ));
                }
            };

            if let Some(command) = usb_command.commands {
                (device.boxed(), command)
            } else {
                return Err(ADBCliError::Standard("no command specified".into()));
            }
        }
        MainCommand::Tcp(tcp_command) => {
            let device = match tcp_command.path_to_private_key {
                Some(pk) => ADBTcpDevice::new_with_custom_private_key(tcp_command.address, pk)?,
                None => ADBTcpDevice::new(tcp_command.address)?,
            };
            (device.boxed(), tcp_command.commands)
        }
        MainCommand::Mdns => {
            let mut service = MDNSDiscoveryService::new()?;

            let (tx, rx) = std::sync::mpsc::channel();
            service.start(tx)?;

            log::info!("Starting mdns discovery...");
            while let Ok(device) = rx.recv() {
                log::info!(
                    "Found device fullname='{}' with ipv4 addresses={:?} and ipv6 addresses={:?}",
                    device.fullname,
                    device.ipv4_addresses(),
                    device.ipv6_addresses()
                );
            }

            return Ok(service.shutdown()?);
        }
    };

    run_command(device, commands)?;

    Ok(())
}
