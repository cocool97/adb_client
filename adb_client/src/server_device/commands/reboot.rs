use crate::{
    models::{AdbServerCommand, RebootType},
    ADBServerDevice, Result,
};

impl ADBServerDevice {
    /// Reboots the device
    pub fn reboot(&mut self, reboot_type: RebootType) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .proxy_connection(AdbServerCommand::Reboot(reboot_type), false)
            .map(|_| ())
    }
}
