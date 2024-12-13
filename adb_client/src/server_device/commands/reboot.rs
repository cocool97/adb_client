use crate::{
    models::{AdbServerCommand, RebootType},
    ADBServerDevice, Result,
};

impl ADBServerDevice {
    /// Reboots the device
    pub fn reboot(&mut self, reboot_type: RebootType) -> Result<()> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;

        self.transport
            .proxy_connection(AdbServerCommand::Reboot(reboot_type), false)
            .map(|_| ())
    }
}
