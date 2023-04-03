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
            None => Self::send_adb_request(&mut self.tcp_stream, AdbCommand::TransportAny)?,
            Some(serial) => Self::send_adb_request(
                &mut self.tcp_stream,
                AdbCommand::TransportSerial(serial.to_string()),
            )?,
        }

        self.proxy_connexion(AdbCommand::Reboot(reboot_type), false)
            .map(|_| ())
    }
}
