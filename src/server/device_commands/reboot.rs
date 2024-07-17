use crate::{
    models::{AdbCommand, RebootType},
    ADBServerDevice, Result,
};

impl ADBServerDevice {
    /// Reboots the device
    pub fn reboot(&mut self, reboot_type: RebootType) -> Result<()> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbCommand::TransportSerial(serial))?;

        self.get_transport()?
            .proxy_connection(AdbCommand::Reboot(reboot_type), false)
            .map(|_| ())
    }
}
