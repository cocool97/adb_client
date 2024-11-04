use crate::{models::AdbServerCommand, ADBServerDevice, Result};

impl ADBServerDevice {
    /// Forward socket connection
    pub fn forward(&mut self, remote: String, local: String) -> Result<()> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial.clone()))?;

        self.get_transport_mut()
            .proxy_connection(AdbServerCommand::Forward(serial, remote, local), false)
            .map(|_| ())
    }
}
