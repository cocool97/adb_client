use crate::{
    models::{AdbCommand, HostFeatures},
    AdbTcpConnection, Result,
};

impl AdbTcpConnection {
    /// Lists available ADB server features.
    pub fn host_features<S: ToString>(&mut self, serial: Option<&S>) -> Result<Vec<HostFeatures>> {
        match serial {
            None => self.send_adb_request(AdbCommand::TransportAny, true)?,
            Some(serial) => {
                self.send_adb_request(AdbCommand::TransportSerial(serial.to_string()), true)?
            }
        }

        let features = self.proxy_connection(AdbCommand::HostFeatures, true, false)?;

        Ok(features
            .split(|x| x.eq(&b','))
            .filter_map(|v| HostFeatures::try_from(v).ok())
            .collect())
    }
}
