use std::fmt::Display;

use super::RebootType;

pub(crate) enum AdbCommand {
    Version,
    Kill,
    Devices,
    DevicesLong,
    TrackDevices,
    HostFeatures,
    // TODO: NOT IMPLEMENTED YET
    // Emulator(u16),
    // Transport(String),
    // TransportUSB,
    // TransportLocal,
    TransportAny,
    TransportSerial(String),
    // Serial((String, String)),
    // USB(String),
    // Local(String),
    // Request(String),
    // GetProduct(String),
    // GetSerialNo(String),
    // GetDevPath(String),
    // GetState(String),
    // Forward((String, String, String)),
    // ForwardNoRebind((String, String, String)),
    // KillForward((String, String)),
    // KillForwardAll(String),
    // ListForward(String),
    ShellCommand(String),
    Shell,
    // Remount,
    // DevPath(String),
    // Tcp(u16),
    // Tcp((u16, String)),
    // Local(String),
    // LocalReserved(String),
    // LocalAbstract(String),
    // LocalFileSystem(String),
    // FrameBuffer,
    // JDWP(u32),
    // TrackJDWP,
    Sync,
    // Reverse(String),
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
            AdbCommand::Shell => match std::env::var("TERM") {
                Ok(term) => write!(f, "shell,TERM={term},raw:"),
                Err(_) => write!(f, "shell,raw:"),
            },
            AdbCommand::HostFeatures => write!(f, "host:features"),
            AdbCommand::Reboot(reboot_type) => {
                write!(f, "reboot:{reboot_type}")
            }
        }
    }
}
