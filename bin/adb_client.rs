use adb_client::{AdbCommandProvider, AdbTcpConnexion};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(
        short = 'a',
        long = "address",
        help = "Sets the listening address of ADB server",
        default_value = "127.0.0.1"
    )]
    pub address: String,
    #[clap(
        short = 'p',
        long = "port",
        help = "Sets the listening port of ADB server",
        default_value = "5037"
    )]
    pub port: u16,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    Version,
    Devices {
        #[clap(short = 'l', long = "long")]
        long: bool,
    },
}

fn main() {
    let opt = Args::parse();

    let connexion = AdbTcpConnexion::new()
        .address(opt.address)
        .unwrap()
        .port(opt.port);

    match opt.command {
        Command::Version => {
            connexion.version().unwrap();
        }
        Command::Devices { long } => {
            if long {
                connexion.devices_long().unwrap();
            } else {
                for device in connexion.devices().unwrap() {
                    println!("{}\t{}", device.identifier, device.state);
                }
            }
        }
    }
}
