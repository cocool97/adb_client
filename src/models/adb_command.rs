pub enum AdbCommand {
    Version,
    Kill,
    Devices,
    DevicesLong,
    TrackDevices,
}

impl ToString for AdbCommand {
    fn to_string(&self) -> String {
        match self {
            AdbCommand::Version => "host:version".into(),
            AdbCommand::Kill => "host:kill".into(),
            AdbCommand::Devices => "host:devices".into(),
            AdbCommand::DevicesLong => "host:devices-l".into(),
            AdbCommand::TrackDevices => "host:track-devices".into(),
        }
    }
}
