use crate::{models::AdbCommand, ADBServer, Result};

impl ADBServer {
    /// Asks the ADB server to quit immediately.
    pub fn kill(&mut self) -> Result<()> {
        self.connect()?
            .proxy_connection(AdbCommand::Kill, false)
            .map(|_| ())
    }
}
