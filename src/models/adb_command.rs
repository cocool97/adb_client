pub enum AdbCommand {
    Version,
    Devices,
    DevicesLong,
}

impl ToString for AdbCommand {
    fn to_string(&self) -> String {
        match self {
            AdbCommand::Version => "host:version".into(),
            AdbCommand::Devices => "host:devices".into(),
            AdbCommand::DevicesLong => "host:devices-l".into(),
        }
    }
}
