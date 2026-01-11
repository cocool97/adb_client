use crate::{
    Connected, Result,
    models::{ADBCommand, ADBHostCommand},
    server::{ADBServer, AdbVersion},
};

impl ADBServer<Connected> {
    /// Gets server's internal version number.
    pub fn version(&mut self) -> Result<AdbVersion> {
        let version = self
            .get_transport()?
            .proxy_connection(&ADBCommand::Host(ADBHostCommand::Version), true)?;

        AdbVersion::try_from(version)
    }
}
