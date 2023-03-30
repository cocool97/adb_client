use crate::{
    models::{AdbCommand, HostFeatures},
    AdbTcpConnexion, Result,
};

impl AdbTcpConnexion {
    /// Lists available ADB server features.
    pub fn host_features<S: ToString + Clone>(
        &mut self,
        serial: Option<S>,
    ) -> Result<Vec<HostFeatures>> {
        match serial {
            None => Self::send_adb_request(&mut self.tcp_stream, AdbCommand::TransportAny)?,
            Some(serial) => Self::send_adb_request(
                &mut self.tcp_stream,
                AdbCommand::TransportSerial(serial.to_string()),
            )?,
        }

        let features = self.proxy_connexion(AdbCommand::HostFeatures, true)?;

        Ok(features
            .split(|x| x.eq(&b','))
            .filter_map(|v| HostFeatures::try_from(v).ok())
            .collect())
    }
}
