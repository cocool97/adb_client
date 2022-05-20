use std::io::Read;

use crate::{models::AdbCommand, AdbTcpConnexion, Device, DeviceLong, Result, RustADBError};

impl AdbTcpConnexion {
    /// Gets a list of connected devices.
    pub fn devices(&mut self) -> Result<Vec<Device>> {
        let devices = self.proxy_connexion(AdbCommand::Devices, true)?;

        let mut vec_devices: Vec<Device> = vec![];
        for device in devices.split(|x| x.eq(&b'\n')) {
            if device.is_empty() {
                break;
            }

            vec_devices.push(Device::try_from(device.to_vec())?);
        }

        Ok(vec_devices)
    }

    /// Gets an extended list of connected devices including the device paths in the state.
    pub fn devices_long(&mut self) -> Result<Vec<DeviceLong>> {
        let devices_long = self.proxy_connexion(AdbCommand::DevicesLong, true)?;

        let mut vec_devices: Vec<DeviceLong> = vec![];
        for device in devices_long.split(|x| x.eq(&b'\n')) {
            if device.is_empty() {
                break;
            }

            vec_devices.push(DeviceLong::try_from(device.to_vec())?);
        }

        Ok(vec_devices)
    }

    /// Tracks new devices showing up.
    // TODO: Change with Generator when feature stabilizes
    pub fn track_devices(&mut self, callback: fn(Device) -> Result<()>) -> Result<()> {
        Self::send_adb_request(&mut self.tcp_stream, AdbCommand::TrackDevices)?;

        loop {
            let length = Self::get_body_length(&mut self.tcp_stream)?;

            if length > 0 {
                let mut body = vec![
                    0;
                    length
                        .try_into()
                        .map_err(|_| RustADBError::ConvertionError)?
                ];
                self.tcp_stream.read_exact(&mut body)?;

                callback(Device::try_from(body)?)?;
            }
        }
    }
}
