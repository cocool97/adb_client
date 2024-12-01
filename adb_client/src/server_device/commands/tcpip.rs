use crate::{models::AdbServerCommand, ADBServerDevice, Result};

impl ADBServerDevice {
    /// Set adb daemon to tcp/ip mode
    pub fn tcpip(&mut self, port: u16) -> Result<()> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;

        self.get_transport_mut()
            .proxy_connection(AdbServerCommand::TcpIp(port), false)
            .map(|_| ())
    }
}
