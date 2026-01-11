use crate::{
    Connected, Result,
    models::{ADBCommand, ADBHostCommand},
    server::ADBServer,
};

impl ADBServer<Connected> {
    /// Reconnect the device
    pub fn reconnect_offline(&mut self) -> Result<()> {
        self.get_transport()?
            .proxy_connection(&ADBCommand::Host(ADBHostCommand::ReconnectOffline), false)
            .map(|_| ())
    }
}
