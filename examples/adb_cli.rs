use adb_client::{ADBServer, DeviceShort, RebootType};
use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{self, Write};
use std::net::SocketAddrV4;
use std::path::Path;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Args {
    #[clap(short = 'a', long = "address", default_value = "127.0.0.1:5037")]
    pub address: SocketAddrV4,
    /// Serial id of a specific device. Every request will be sent to this device.
    #[clap(short = 's', long = "serial")]
    pub serial: Option<String>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    #[clap(flatten)]
    LocalCommand(LocalCommand),
    #[clap(flatten)]
    HostCommand(HostCommand),
}

#[derive(Parser, Debug)]
pub enum LocalCommand {
    /// List available server features.
    HostFeatures,
    /// Push a file on device
    Push { filename: String, path: String },
    /// Pull a file from device
    Pull { path: String, filename: String },
    /// List a directory on device
    List { path: String },
    /// Stat a file specified on device
    Stat { path: String },
    /// Spawn an interactive shell or run a list of commands on the device
    Shell { command: Vec<String> },
    /// Reboot the device
    Reboot {
        #[clap(subcommand)]
        sub_command: RebootTypeCommand,
    },
    /// Get framebuffer of device
    Framebuffer { path: String },
}

#[derive(Parser, Debug)]
pub enum HostCommand {
    /// Print current ADB version.
    Version,
    /// Ask ADB server to quit immediately.
    Kill,
    /// List connected devices.
    Devices {
        #[clap(short = 'l', long = "long")]
        long: bool,
    },
    /// Track new devices showing up.
    TrackDevices,
    /// Pair device with a given code
    Pair { address: SocketAddrV4, code: u32 },
    /// Connect device over WI-FI
    Connect { address: SocketAddrV4 },
    /// Disconnect device over WI-FI
    Disconnect { address: SocketAddrV4 },
}

#[derive(Parser, Debug)]
pub enum RebootTypeCommand {
    System,
    Bootloader,
    Recovery,
    Sideload,
    SideloadAutoReboot,
}

impl From<RebootTypeCommand> for RebootType {
    fn from(value: RebootTypeCommand) -> Self {
        match value {
            RebootTypeCommand::System => RebootType::System,
            RebootTypeCommand::Bootloader => RebootType::Bootloader,
            RebootTypeCommand::Recovery => RebootType::Recovery,
            RebootTypeCommand::Sideload => RebootType::Sideload,
            RebootTypeCommand::SideloadAutoReboot => RebootType::SideloadAutoReboot,
        }
    }
}

fn main() -> Result<()> {
    let opt = Args::parse();

    let mut adb_server = ADBServer::new(opt.address);

    match opt.command {
        Command::LocalCommand(local) => {
            let mut device = match opt.serial {
                Some(serial) => adb_server.get_device_by_name(serial)?,
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
        Command::HostCommand(host) => match host {
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
        },
    }

    Ok(())
}
