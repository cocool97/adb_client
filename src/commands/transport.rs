use crate::{models::AdbCommand, AdbTcpConnexion, Result};

impl AdbTcpConnexion {
    /// Asks ADB server to switch the connection to either the device or emulator connect to/running on the host. Will fail if there is more than one such device/emulator available.
    pub fn transport_any(&self) -> Result<()> {
        self.proxy_connexion(AdbCommand::TransportAny, false)
            .map(|_| ())
    }
}
