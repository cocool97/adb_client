use super::RebootType;

pub enum AdbCommand {
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

impl ToString for AdbCommand {
    fn to_string(&self) -> String {
        match self {
            AdbCommand::Version => "host:version".into(),
            AdbCommand::Kill => "host:kill".into(),
            AdbCommand::Devices => "host:devices".into(),
            AdbCommand::DevicesLong => "host:devices-l".into(),
            AdbCommand::Sync => "sync:".into(),
            AdbCommand::TrackDevices => "host:track-devices".into(),
            AdbCommand::TransportAny => "host:transport-any".into(),
            AdbCommand::TransportSerial(serial) => format!("host:transport:{serial}"),
            AdbCommand::ShellCommand(command) => match std::env::var("TERM") {
                Ok(term) => format!("shell,TERM={term},raw:{command}"),
                Err(_) => format!("shell,raw:{command}"),
            },
            AdbCommand::Shell => match std::env::var("TERM") {
                Ok(term) => format!("shell,TERM={term},raw:"),
                Err(_) => "shell,raw:".into(),
            },
            AdbCommand::HostFeatures => "host:features".into(),
            AdbCommand::Reboot(reboot_type) => {
                format!("reboot:{reboot_type}")
            }
        }
    }
}
