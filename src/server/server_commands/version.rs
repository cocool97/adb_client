use crate::{models::AdbCommand, ADBServer, AdbVersion, Result};

impl ADBServer {
    /// Gets server's internal version number.
    pub fn version(&mut self) -> Result<AdbVersion> {
        let version = self
            .connect()?
            .proxy_connection(AdbCommand::Version, true)?;

        AdbVersion::try_from(version)
    }
}
