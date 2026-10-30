use crate::{
    Result,
    models::{ADBCommand, ADBLocalCommand},
    server_device::ADBServerDevice,
};

impl ADBServerDevice {
    /// Reverse socket connection
    pub fn reverse(&self, remote: String, local: String) -> Result<()> {
        let transport = self.connect_to_device()?;

        transport
            .proxy_connection(
                &ADBCommand::Local(ADBLocalCommand::Reverse(remote, local)),
                false,
            )
            .map(|_| ())
    }

    /// Remove a previously applied reverse rule by its remote endpoint.
    pub fn reverse_remove(&self, remote: String) -> Result<()> {
        let transport = self.connect_to_device()?;

        transport
            .proxy_connection(
                &ADBCommand::Local(ADBLocalCommand::ReverseRemove(remote)),
                false,
            )
            .map(|_| ())
    }

    /// Remove all reverse rules
    pub fn reverse_remove_all(&self) -> Result<()> {
        let transport = self.connect_to_device()?;

        transport
            .proxy_connection(&ADBCommand::Local(ADBLocalCommand::ReverseRemoveAll), false)
            .map(|_| ())
    }
}
