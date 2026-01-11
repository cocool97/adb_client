use crate::{
    Connected, Result,
    models::{ADBCommand, ADBHostCommand},
    server::ADBServer,
};

impl ADBServer<Connected> {
    /// Asks the ADB server to quit immediately.
    pub fn kill(mut self) -> Result<()> {
        self.get_transport()?
            .proxy_connection(&ADBCommand::Host(ADBHostCommand::Kill), false)
            .map(|_| ())
    }
}
