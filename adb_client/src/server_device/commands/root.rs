use crate::{
    Result,
    models::{ADBCommand, ADBLocalCommand},
    server_device::ADBServerDevice,
};

impl ADBServerDevice {
    /// Restart adb daemon with root permissions
    pub fn root(&self) -> Result<()> {
        // adbd restarts, and may then advertise different features
        self.invalidate_features();
        let transport = self.connect_to_device()?;

        transport
            .proxy_connection(&ADBCommand::Local(ADBLocalCommand::Root), false)
            .map(|_| ())
    }
}
