use crate::{ADBServer, Result, models::AdbServerCommand};

impl ADBServer {
    /// Asks the ADB server to quit immediately.
    pub fn kill(&mut self) -> Result<()> {
        self.connect()?
            .proxy_connection(AdbServerCommand::Kill, false)
            .map(|_| ())
    }
}
