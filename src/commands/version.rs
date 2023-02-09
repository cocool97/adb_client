use crate::{models::AdbCommand, AdbTcpConnexion, AdbVersion, Result};

impl AdbTcpConnexion {
    /// Gets server's internal version number.
    pub fn version(&mut self) -> Result<AdbVersion> {
        let version = self.proxy_connexion(AdbCommand::Version, true)?;

        AdbVersion::try_from(version)
    }
}
