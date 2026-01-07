use crate::{server_device::ADBServerDevice, Result, models::{ADBCommand, ADBLocalCommand}};

impl ADBServerDevice {
    /// Restart adb daemon with root permissions
    pub fn root(&mut self) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .proxy_connection(&ADBCommand::Local(ADBLocalCommand::Root), false)
            .map(|_| ())
    }
}
