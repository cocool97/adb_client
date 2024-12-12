use crate::{models::AdbServerCommand, ADBProtoPort, ADBServerDevice, Result};

impl ADBServerDevice {
    /// Forward socket connection
    pub fn forward(&mut self, remote: ADBProtoPort, local: ADBProtoPort) -> Result<()> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial.clone()))?;

        self.get_transport_mut()
            .proxy_connection(AdbServerCommand::Forward(remote, local), false)
            .map(|_| ())
    }

    /// Remove all previously applied forward rules
    pub fn forward_remove_all(&mut self) -> Result<()> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial.clone()))?;

        self.get_transport_mut()
            .proxy_connection(AdbServerCommand::ForwardRemoveAll, false)
            .map(|_| ())
    }
}
