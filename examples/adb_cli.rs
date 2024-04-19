use std::fs::File;
use std::net::Ipv4Addr;
use std::path::Path;

use adb_client::{AdbTcpConnection, Device, RebootType, RustADBError};
use clap::Parser;

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
    /// Prints current ADB version.
    Version,
    /// Asks ADB server to quit immediately.
    Kill,
    /// List connected devices.
    Devices {
        #[clap(short = 'l', long = "long")]
        long: bool,
    },
    /// Tracks new devices showing up.
    TrackDevices,
    /// Lists available server features.
    HostFeatures,
    /// Pushes 'filename' to the 'path' on device
    Push { filename: String, path: String },
    /// Pushes 'path' on the device to 'filename'
    Pull { path: String, filename: String },
    /// List files for 'path' on device
    List { path: String },
    /// Stat file specified as 'path' on device
    Stat { path: String },
    /// Run 'command' in a shell on the device, and return its output and error streams.
    Shell { command: Vec<String> },
    /// Reboots the device
    Reboot {
        #[clap(subcommand)]
        sub_command: RebootTypeCommand,
    },
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

fn main() -> Result<(), RustADBError> {
    let opt = Args::parse();

    let mut connection = AdbTcpConnection::new(opt.address, opt.port)?;

    match opt.command {
        Command::Version => {
            let version = connection.version()?;
            println!("Android Debug Bridge version {}", version);
            println!("Package version {}-rust", std::env!("CARGO_PKG_VERSION"));
        }
        Command::Kill => {
            connection.kill()?;
        }
        Command::Devices { long } => {
            if long {
                println!("List of devices attached (extended)");
                for device in connection.devices_long()? {
                    println!("{}", device);
                }
            } else {
                println!("List of devices attached");
                for device in connection.devices()? {
                    println!("{}", device);
                }
            }
        }
        Command::TrackDevices => {
            let callback = |device: Device| {
                println!("{}", device);
                Ok(())
            };
            println!("Live list of devices attached");
            connection.track_devices(callback)?;
        }
        Command::Pull { path, filename } => {
            let mut output = File::create(Path::new(&filename)).unwrap(); // TODO: Better error handling
            connection.recv(opt.serial, &path, &mut output)?;
            println!("Downloaded {path} as {filename}");
        }
        Command::Push { filename, path } => {
            let mut input = File::open(Path::new(&filename)).unwrap(); // TODO: Better error handling
            connection.send(opt.serial, &mut input, &path)?;
            println!("Uploaded {filename} to {path}");
        }
        Command::List { path } => {
            connection.list(opt.serial, path)?;
        }
        Command::Stat { path } => {
            let stat_response = connection.stat(opt.serial, path)?;
            println!("{}", stat_response);
        }
        Command::Shell { command } => {
            if command.is_empty() {
                #[cfg(unix)]
                connection.shell(&opt.serial)?;
                if cfg!(windows) {
                    println!("Interative shell not supported on Windows");
                }
            } else {
                connection.shell_command(&opt.serial, command)?;
            }
        }
        Command::HostFeatures => {
            println!("Available host features");
            for feature in connection.host_features(&opt.serial)? {
                println!("- {}", feature);
            }
        }
        Command::Reboot { sub_command } => {
            println!("Reboots device");
            connection.reboot(&opt.serial, sub_command.into())?
        }
    }

    Ok(())
}
