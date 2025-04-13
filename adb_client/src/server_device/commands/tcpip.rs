use crate::{ADBServerDevice, Result, models::AdbServerCommand};

impl ADBServerDevice {
    /// Set adb daemon to tcp/ip mode
    pub fn tcpip(&mut self, port: u16) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .proxy_connection(AdbServerCommand::TcpIp(port), false)
            .map(|_| ())
    }
}
