use crate::{models::AdbServerCommand, ADBServerDevice, Result};

impl ADBServerDevice {
    /// Reconnect device
    pub fn reconnect(&mut self) -> Result<()> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;

        self.transport
            .proxy_connection(AdbServerCommand::Reconnect, false)
            .map(|_| ())
    }
}
