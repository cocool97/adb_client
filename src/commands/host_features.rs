use crate::{
    models::{AdbCommand, HostFeatures},
    AdbTcpConnexion, Result,
};

impl AdbTcpConnexion {
    /// Lists available ADB server features.
    pub fn host_features<S: ToString>(&mut self, serial: &Option<S>) -> Result<Vec<HostFeatures>> {
        match serial {
            None => self.send_adb_request(AdbCommand::TransportAny)?,
            Some(serial) => {
                self.send_adb_request(AdbCommand::TransportSerial(serial.to_string()))?
            }
        }

        let features = self.proxy_connexion(AdbCommand::HostFeatures, true)?;

        Ok(features
            .split(|x| x.eq(&b','))
            .filter_map(|v| HostFeatures::try_from(v).ok())
            .collect())
    }
}
