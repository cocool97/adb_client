use crate::{
    Result,
    models::{ADBCommand, ADBHostCommand, HostFeatures},
    server_device::{ADBServerDevice, adb_server_device::CachedFeatures},
};

impl ADBServerDevice {
    /// Lists available ADB server features.
    ///
    /// Features are fetched from the device once, then cached by this instance. Cache is refreshed if
    /// `identifier` or `transport_id` change, or after commands restarting adbd (reboot, root, tcpip, usb,
    /// reconnect). When no device is selected (`transport-any`), the cache assumes the same device stays
    /// connected.
    pub fn host_features(&self) -> Result<Vec<HostFeatures>> {
        if let Some(cached) = self.cached_features.lock()?.as_ref()
            && cached.identifier == self.identifier
            && cached.transport_id == self.transport_id
        {
            return Ok(cached.features.clone());
        }

        let features: Vec<HostFeatures> = self
            .connect_to_device()?
            .proxy_connection(&ADBCommand::Host(ADBHostCommand::HostFeatures), true)?
            .split(|x| x.eq(&b','))
            .filter_map(|v| match HostFeatures::try_from(v) {
                Ok(feature) => Some(feature),
                Err(err) => {
                    log::warn!("cannot create new feature: {err}");
                    None
                }
            })
            .collect();

        *self.cached_features.lock()? = Some(CachedFeatures {
            identifier: self.identifier.clone(),
            transport_id: self.transport_id,
            features: features.clone(),
        });

        Ok(features)
    }
}
