use crate::{
    Result,
    models::{ADBCommand, ADBLocalCommand},
    server_device::ADBServerDevice,
};

impl ADBServerDevice {
    /// Set adb daemon to usb mode
    pub fn usb(&self) -> Result<()> {
        // adbd restarts, and may then advertise different features
        self.invalidate_features();
        let transport = self.connect_to_device()?;
        transport
            .proxy_connection(&ADBCommand::Local(ADBLocalCommand::Usb), false)
            .map(|_| ())
    }
}
