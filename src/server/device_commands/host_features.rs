use crate::{
    models::{AdbCommand, HostFeatures},
    ADBServerDevice, Result,
};

impl ADBServerDevice {
    /// Lists available ADB server features.
    pub fn host_features(&mut self) -> Result<Vec<HostFeatures>> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbCommand::TransportSerial(serial))?;

        let features = self
            .get_transport()?
            .proxy_connection(AdbCommand::HostFeatures, true)?;

        Ok(features
            .split(|x| x.eq(&b','))
            .filter_map(|v| HostFeatures::try_from(v).ok())
            .collect())
    }
}
