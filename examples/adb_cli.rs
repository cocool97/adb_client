use adb_client::{ADBServer, DeviceShort, RebootType};
use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{self, Write};
use std::net::Ipv4Addr;
use std::path::Path;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Args {
    /// Sets the listening address of ADB server
    #[clap(short = 'a', long = "address", default_value = "127.0.0.1")]
    pub address: Ipv4Addr,
    /// Sets the listening port of ADB server
    #[clap(short = 'p', long = "port", default_value = "5037")]
    pub port: u16,
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

    let mut adb_server = ADBServer::new(opt.address, opt.port);

    match opt.command {
        Command::LocalCommand(local) => {
            let mut device = adb_server.get_device(opt.serial.to_owned())?;
            match local {
                LocalCommand::Pull { path, filename } => {
                    let mut output = File::create(Path::new(&filename))?;
                    device.recv(opt.serial.as_ref(), &path, &mut output)?;
                    println!("Downloaded {path} as {filename}");
                }
                LocalCommand::Push { filename, path } => {
                    let mut input = File::open(Path::new(&filename))?;
                    device.send(opt.serial.as_ref(), &mut input, &path)?;
                    println!("Uploaded {filename} to {path}");
                }
                LocalCommand::List { path } => {
                    device.list(opt.serial.as_ref(), path)?;
                }
                LocalCommand::Stat { path } => {
                    let stat_response = device.stat(opt.serial, path)?;
                    println!("{}", stat_response);
                }
                LocalCommand::Shell { command } => {
                    if command.is_empty() {
                        device.shell(opt.serial.as_ref())?;
                    } else {
                        let stdout = device.shell_command(opt.serial.as_ref(), command)?;
                        io::stdout().write_all(&stdout)?;
                    }
                }
                LocalCommand::HostFeatures => {
                    println!("Available host features");
                    for feature in device.host_features(opt.serial.as_ref())? {
                        println!("- {}", feature);
                    }
                }
                LocalCommand::Reboot { sub_command } => {
                    println!("Reboots device");
                    device.reboot(opt.serial.as_ref(), sub_command.into())?
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
        },
    }

    Ok(())
}
