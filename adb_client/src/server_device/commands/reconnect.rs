use crate::{Result, server::AdbServerCommand, server_device::ADBServerDevice};

impl ADBServerDevice {
    /// Reconnect device
    pub fn reconnect(&mut self) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .proxy_connection(AdbServerCommand::Reconnect, false)
            .map(|_| ())
    }
}
