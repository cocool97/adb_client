use crate::{models::AdbServerCommand, ADBServerDevice, Result};

impl ADBServerDevice {
    /// Set adb daemon to usb mode
    pub fn usb(&mut self) -> Result<()> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;

        self.transport
            .proxy_connection(AdbServerCommand::Usb, false)
            .map(|_| ())
    }
}
