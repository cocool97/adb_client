use crate::{Result, server::AdbServerCommand, server_device::ADBServerDevice};

impl ADBServerDevice {
    /// Disable verity on the device
    pub fn disable_verity(&mut self) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .send_adb_request(&AdbServerCommand::DisableVerity)
    }

    /// Enable verity on the device
    pub fn enable_verity(&mut self) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .send_adb_request(&AdbServerCommand::EnableVerity)
    }
}
