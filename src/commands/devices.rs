use std::io::Read;

use crate::{models::AdbCommand, AdbTcpConnection, Device, DeviceLong, Result, RustADBError};

impl AdbTcpConnection {
    /// Gets a list of connected devices.
    pub fn devices(&mut self) -> Result<Vec<Device>> {
        let devices = self.proxy_connection(AdbCommand::Devices, true, true)?;

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
        let devices_long = self.proxy_connection(AdbCommand::DevicesLong, true, true)?;

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
    pub fn track_devices(&mut self, callback: impl Fn(Device) -> Result<()>) -> Result<()> {
        self.send_adb_request(AdbCommand::TrackDevices, true)?;

        loop {
            let length = self.get_hex_body_length()?;

            if length > 0 {
                let mut body = vec![
                    0;
                    length
                        .try_into()
                        .map_err(|_| RustADBError::ConversionError)?
                ];
                self.get_connection()?.read_exact(&mut body)?;

                callback(Device::try_from(body)?)?;
            }
        }
    }
}
