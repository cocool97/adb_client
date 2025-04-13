use crate::{ADBServerDevice, Result, models::AdbServerCommand};

impl ADBServerDevice {
    /// Set adb daemon to usb mode
    pub fn usb(&mut self) -> Result<()> {
        self.set_serial_transport()?;
        self.transport
            .proxy_connection(AdbServerCommand::Usb, false)
            .map(|_| ())
    }
}
