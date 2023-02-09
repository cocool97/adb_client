use crate::{
    models::{AdbCommand, RebootType},
    AdbTcpConnexion, Result,
};

impl AdbTcpConnexion {
    /// Reboots the device
    pub fn reboot(&mut self, serial: Option<String>, reboot_type: RebootType) -> Result<()> {
        match serial {
            None => Self::send_adb_request(&mut self.tcp_stream, AdbCommand::TransportAny)?,
            Some(serial) => {
                Self::send_adb_request(&mut self.tcp_stream, AdbCommand::TransportSerial(serial))?
            }
        }

        self.proxy_connexion(AdbCommand::Reboot(reboot_type), false)
            .map(|_| ())
    }
}
