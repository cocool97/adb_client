use crate::{
    models::{AdbCommand, RebootType},
    ADBServerDevice, Result,
};

impl ADBServerDevice {
    /// Reboots the device
    pub fn reboot<S: ToString>(
        &mut self,
        serial: Option<&S>,
        reboot_type: RebootType,
    ) -> Result<()> {
        match serial {
            None => self.connect()?.send_adb_request(AdbCommand::TransportAny)?,
            Some(serial) => self
                .connect()?
                .send_adb_request(AdbCommand::TransportSerial(serial.to_string()))?,
        }

        self.get_transport()?
            .proxy_connection(AdbCommand::Reboot(reboot_type), false)
            .map(|_| ())
    }
}
