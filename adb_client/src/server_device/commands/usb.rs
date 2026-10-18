use crate::{
    Result,
    models::{ADBCommand, ADBLocalCommand},
    server_device::ADBServerDevice,
};

impl ADBServerDevice {
    /// Set adb daemon to usb mode
    pub fn usb(&mut self) -> Result<()> {
        // adbd restarts, and may then advertise different features
        self.invalidate_features();
        self.set_serial_transport()?;
        self.transport
            .proxy_connection(&ADBCommand::Local(ADBLocalCommand::Usb), false)
            .map(|_| ())
    }
}
