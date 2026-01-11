use crate::{
    Connected, Result,
    models::{ADBCommand, ADBHostCommand},
    server::{ADBServer, models::ServerStatus},
};

impl ADBServer<Connected> {
    /// Check ADB server status
    pub fn server_status(&mut self) -> Result<ServerStatus> {
        let status = self
            .get_transport()?
            .proxy_connection(&ADBCommand::Host(ADBHostCommand::ServerStatus), true)?;

        ServerStatus::try_from(status)
    }
}
