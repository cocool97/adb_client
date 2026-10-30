use crate::{
    Result,
    models::{ADBCommand, ADBLocalCommand},
    server_device::ADBServerDevice,
};

impl ADBServerDevice {
    /// Forward socket connection
    pub fn forward(&self, remote: String, local: String) -> Result<()> {
        let transport = self.connect_to_device()?;

        transport
            .proxy_connection(
                &ADBCommand::Local(ADBLocalCommand::Forward(remote, local)),
                false,
            )
            .map(|_| ())
    }

    /// Remove a previously applied forward rule by its local endpoint.
    pub fn forward_remove(&self, local: String) -> Result<()> {
        let transport = self.connect_to_device()?;

        transport
            .proxy_connection(
                &ADBCommand::Local(ADBLocalCommand::ForwardRemove(local)),
                false,
            )
            .map(|_| ())
    }

    /// Remove all previously applied forward rules
    pub fn forward_remove_all(&self) -> Result<()> {
        let transport = self.connect_to_device()?;

        transport
            .proxy_connection(&ADBCommand::Local(ADBLocalCommand::ForwardRemoveAll), false)
            .map(|_| ())
    }
}
