use crate::{
    models::{AdbCommand, RebootType},
    AdbTcpConnexion, Result,
};

impl AdbTcpConnexion {
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

        self.proxy_connexion(AdbCommand::Reboot(reboot_type), false)
            .map(|_| ())
    }
}
