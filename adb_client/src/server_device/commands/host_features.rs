use crate::{
    models::{AdbServerCommand, HostFeatures},
    ADBServerDevice, Result,
};

impl ADBServerDevice {
    /// Lists available ADB server features.
    pub fn host_features(&mut self) -> Result<Vec<HostFeatures>> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;

        let features = self
            .transport
            .proxy_connection(AdbServerCommand::HostFeatures, true)?;

        Ok(features
            .split(|x| x.eq(&b','))
            .filter_map(|v| HostFeatures::try_from(v).ok())
            .collect())
    }
}
