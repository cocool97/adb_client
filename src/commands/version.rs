use crate::{models::AdbCommand, AdbTcpConnection, AdbVersion, Result};

impl AdbTcpConnection {
    /// Gets server's internal version number.
    pub fn version(&mut self) -> Result<AdbVersion> {
        let version = self.proxy_connection(AdbCommand::Version, true, true)?;

        AdbVersion::try_from(version)
    }
}
