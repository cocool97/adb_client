use crate::{models::AdbServerCommand, ADBServer, AdbVersion, Result};

impl ADBServer {
    /// Gets server's internal version number.
    pub fn version(&mut self) -> Result<AdbVersion> {
        let version = self
            .connect()?
            .proxy_connection(AdbServerCommand::Version, true)?;

        AdbVersion::try_from(version)
    }
}
