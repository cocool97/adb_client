use crate::{
    Result,
    adb_transport::Connected,
    models::{ADBCommand, ADBLocalCommand},
    server_device::ADBServerDevice,
};

impl ADBServerDevice<Connected> {
    /// Set adb daemon to usb mode
    pub fn usb(&mut self) -> Result<()> {
        self.set_serial_transport()?;
        self.transport
            .proxy_connection(&ADBCommand::Local(ADBLocalCommand::Usb), false)
            .map(|_| ())
    }
}
