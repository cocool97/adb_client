use crate::{models::AdbServerCommand, server::models::ServerStatus, ADBServer, Result};

impl ADBServer {
    /// Check ADB server status
    pub fn server_status(&mut self) -> Result<ServerStatus> {
        let status = self
            .connect()?
            .proxy_connection(AdbServerCommand::ServerStatus, true)?;

        ServerStatus::try_from(status)
    }
}
