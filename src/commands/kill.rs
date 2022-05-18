use crate::{models::AdbCommand, AdbTcpConnexion, Result};

impl AdbTcpConnexion {
    /// Asks the ADB server to quit immediately.
    pub fn kill(&self) -> Result<()> {
        self.proxy_connexion(AdbCommand::Kill, false).map(|_| ())
    }
}
