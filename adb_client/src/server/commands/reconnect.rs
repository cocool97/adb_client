use crate::{models::AdbServerCommand, ADBServer, Result};

impl ADBServer {
    /// Reconnect the device
    pub fn reconnect_offline(&mut self) -> Result<()> {
        self.connect()?
            .proxy_connection(AdbServerCommand::ReconnectOffline, false)
            .map(|_| ())
    }
}
