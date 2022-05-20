mod opt;

use adb_client::{AdbTcpConnexion, Device};
use anyhow::Result;
use clap::Parser;
use opt::{Args, Command};

fn main() -> Result<()> {
    let opt = Args::parse();

    let mut connexion = AdbTcpConnexion::new(opt.address, opt.port)?;

    match opt.command {
        Command::Version => {
            let version = connexion.version()?;
            println!("Android Debug Bridge version {}", version);
            println!("Package version {}-rust", std::env!("CARGO_PKG_VERSION"));
        }
        Command::Kill => {
            connexion.kill()?;
        }
        Command::Devices { long } => {
            if long {
                println!("List of devices attached (extended)");
                for device in connexion.devices_long()? {
                    println!("{}", device);
                }
            } else {
                println!("List of devices attached");
                for device in connexion.devices()? {
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
            connexion.track_devices(callback)?;
        }
        Command::Shell { command } => {
            if command.is_empty() {
                connexion.shell(opt.serial)?;
            } else {
                connexion.shell_command(opt.serial, command)?;
            }
        }
        Command::HostFeatures => {
            println!("Available host features");
            for feature in connexion.host_features()? {
                println!("- {}", feature);
            }
        }
    }

    Ok(())
}
