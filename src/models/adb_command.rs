pub enum AdbCommand {
    Version,
    Kill,
    Devices,
    DevicesLong,
    TrackDevices,
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
    // Sync,
    // Reverse(String)
}

impl ToString for AdbCommand {
    fn to_string(&self) -> String {
        match self {
            AdbCommand::Version => "host:version".into(),
            AdbCommand::Kill => "host:kill".into(),
            AdbCommand::Devices => "host:devices".into(),
            AdbCommand::DevicesLong => "host:devices-l".into(),
            AdbCommand::TrackDevices => "host:track-devices".into(),
            AdbCommand::TransportAny => "host:transport-any".into(),
            AdbCommand::TransportSerial(serial) => format!("host:transport:{}", serial),
            AdbCommand::ShellCommand(command) => format!("shell:{}", command),
            AdbCommand::Shell => "shell:".into(),
        }
    }
}
