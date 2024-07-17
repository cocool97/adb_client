use crate::{
    models::{AdbCommand, HostFeatures},
    ADBServerDevice, Result,
};

impl ADBServerDevice {
    /// Lists available ADB server features.
    pub fn host_features<S: ToString>(&mut self, serial: Option<&S>) -> Result<Vec<HostFeatures>> {
        match serial {
            None => self.connect()?.send_adb_request(AdbCommand::TransportAny)?,
            Some(serial) => self
                .connect()?
                .send_adb_request(AdbCommand::TransportSerial(serial.to_string()))?,
        }

        let features = self
            .get_transport()?
            .proxy_connection(AdbCommand::HostFeatures, true)?;

        Ok(features
            .split(|x| x.eq(&b','))
            .filter_map(|v| HostFeatures::try_from(v).ok())
            .collect())
    }
}
