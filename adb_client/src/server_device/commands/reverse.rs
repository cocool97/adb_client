use crate::{Result, server::AdbServerCommand, server_device::ADBServerDevice};

impl ADBServerDevice {
    /// Reverse socket connection
    pub fn reverse(&mut self, remote: String, local: String) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .proxy_connection(AdbServerCommand::Reverse(remote, local), false)
            .map(|_| ())
    }

    /// Remove all reverse rules
    pub fn reverse_remove_all(&mut self) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .proxy_connection(AdbServerCommand::ReverseRemoveAll, false)
            .map(|_| ())
    }
}
