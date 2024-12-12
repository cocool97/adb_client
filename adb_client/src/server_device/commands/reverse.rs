use crate::{models::AdbServerCommand, ADBProtoPort, ADBServerDevice, Result};

impl ADBServerDevice {
    /// Reverse socket connection
    pub fn reverse(&mut self, remote: ADBProtoPort, local: ADBProtoPort) -> Result<()> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;

        self.get_transport_mut()
            .proxy_connection(AdbServerCommand::Reverse(remote, local), false)
            .map(|_| ())
    }

    /// Remove all reverse rules
    pub fn reverse_remove_all(&mut self) -> Result<()> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial.clone()))?;

        self.get_transport_mut()
            .proxy_connection(AdbServerCommand::ReverseRemoveAll, false)
            .map(|_| ())
    }
}
