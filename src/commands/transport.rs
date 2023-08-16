use crate::{models::AdbCommand, AdbTcpConnection, Result};

impl AdbTcpConnection {
    /// Asks ADB server to switch the connection to either the device or emulator connect to/running on the host. Will fail if there is more than one such device/emulator available.
    pub fn transport_any(&mut self) -> Result<()> {
        self.proxy_connection(AdbCommand::TransportAny, false)
            .map(|_| ())
    }
}
