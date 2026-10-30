use crate::{
    Result,
    models::{ADBCommand, ADBLocalCommand},
    server_device::ADBServerDevice,
};

impl ADBServerDevice {
    /// Reconnect device
    pub fn reconnect(&self) -> Result<()> {
        // adbd restarts, and may then advertise different features
        self.invalidate_features();
        let transport = self.connect_to_device()?;

        transport
            .proxy_connection(&ADBCommand::Local(ADBLocalCommand::Reconnect), false)
            .map(|_| ())
    }
}
