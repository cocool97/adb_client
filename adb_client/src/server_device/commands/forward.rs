use crate::{ADBServerDevice, Result, models::AdbServerCommand};

impl ADBServerDevice {
    /// Forward socket connection
    pub fn forward(&mut self, remote: String, local: String) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .proxy_connection(AdbServerCommand::Forward(remote, local), false)
            .map(|_| ())
    }

    /// Remove all previously applied forward rules
    pub fn forward_remove_all(&mut self) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .proxy_connection(AdbServerCommand::ForwardRemoveAll, false)
            .map(|_| ())
    }
}
