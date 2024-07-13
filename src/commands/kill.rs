use crate::{models::AdbCommand, AdbTcpConnection, Result};

impl AdbTcpConnection {
    /// Asks the ADB server to quit immediately.
    pub fn kill(&mut self) -> Result<()> {
        self.proxy_connection(AdbCommand::Kill, false, true).map(|_| ())
    }
}
