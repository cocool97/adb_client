use std::fmt::Display;

use super::RebootType;
use std::net::SocketAddrV4;

pub(crate) enum AdbCommand {
    Version,
    Kill,
    Devices,
    DevicesLong,
    TrackDevices,
    HostFeatures,
    Connect(SocketAddrV4),
    Disconnect(SocketAddrV4),
    Pair(SocketAddrV4, u32),
    TransportAny,
    TransportSerial(String),
    ShellCommand(String),
    #[cfg(feature = "termios")]
    Shell,
    FrameBuffer,
    Sync,
    Reboot(RebootType),
}

impl Display for AdbCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdbCommand::Version => write!(f, "host:version"),
            AdbCommand::Kill => write!(f, "host:kill"),
            AdbCommand::Devices => write!(f, "host:devices"),
            AdbCommand::DevicesLong => write!(f, "host:devices-l"),
            AdbCommand::Sync => write!(f, "sync:"),
            AdbCommand::TrackDevices => write!(f, "host:track-devices"),
            AdbCommand::TransportAny => write!(f, "host:transport-any"),
            AdbCommand::TransportSerial(serial) => write!(f, "host:transport:{serial}"),
            AdbCommand::ShellCommand(command) => match std::env::var("TERM") {
                Ok(term) => write!(f, "shell,TERM={term},raw:{command}"),
                Err(_) => write!(f, "shell,raw:{command}"),
            },
            #[cfg(feature = "termios")]
            AdbCommand::Shell => match std::env::var("TERM") {
                Ok(term) => write!(f, "shell,TERM={term},raw:"),
                Err(_) => write!(f, "shell,raw:"),
            },
            AdbCommand::HostFeatures => write!(f, "host:features"),
            AdbCommand::Reboot(reboot_type) => {
                write!(f, "reboot:{reboot_type}")
            }
            AdbCommand::Connect(addr) => write!(f, "host:connect:{}", addr),
            AdbCommand::Disconnect(addr) => write!(f, "host:disconnect:{}", addr),
            AdbCommand::Pair(addr, code) => {
                write!(f, "host:pair:{code}:{}", addr)
            }
            AdbCommand::FrameBuffer => write!(f, "framebuffer:"),
        }
    }
}
