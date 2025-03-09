use crate::{models::AdbServerCommand, ADBServerDevice, Result};

impl ADBServerDevice {
    /// Reconnect device
    pub fn reconnect(&mut self) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .proxy_connection(AdbServerCommand::Reconnect, false)
            .map(|_| ())
    }
}
