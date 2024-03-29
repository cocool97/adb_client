use crate::{
    models::{AdbCommand, RebootType},
    AdbTcpConnection, Result,
};

impl AdbTcpConnection {
    /// Reboots the device
    pub fn reboot<S: ToString>(
        &mut self,
        serial: &Option<S>,
        reboot_type: RebootType,
    ) -> Result<()> {
        match serial {
            None => self.send_adb_request(AdbCommand::TransportAny)?,
            Some(serial) => {
                self.send_adb_request(AdbCommand::TransportSerial(serial.to_string()))?
            }
        }

        self.proxy_connection(AdbCommand::Reboot(reboot_type), false)
            .map(|_| ())
    }
}
