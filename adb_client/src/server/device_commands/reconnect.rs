use crate::{
    models::AdbServerCommand,
    ADBServerDevice, Result,
};

impl ADBServerDevice {
    /// reconnect device
    pub fn reconnect(&mut self) -> Result<()> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;

        self.get_transport_mut()
            .proxy_connection(AdbServerCommand::Reconnect, false)
            .map(|_| ())
    }
}
