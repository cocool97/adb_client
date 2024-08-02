use std::io::Read;

use crate::{
    models::{AdbCommand, DeviceShort},
    ADBServer, ADBServerDevice, DeviceLong, Result, RustADBError,
};

impl ADBServer {
    /// Gets a list of connected devices.
    pub fn devices(&mut self) -> Result<Vec<DeviceShort>> {
        let devices = self
            .connect()?
            .proxy_connection(AdbCommand::Devices, true)?;

        let mut vec_devices: Vec<DeviceShort> = vec![];
        for device in devices.split(|x| x.eq(&b'\n')) {
            if device.is_empty() {
                break;
            }

            vec_devices.push(DeviceShort::try_from(device.to_vec())?);
        }

        Ok(vec_devices)
    }

    /// Gets an extended list of connected devices including the device paths in the state.
    pub fn devices_long(&mut self) -> Result<Vec<DeviceLong>> {
        let devices_long = self
            .connect()?
            .proxy_connection(AdbCommand::DevicesLong, true)?;

        let mut vec_devices: Vec<DeviceLong> = vec![];
        for device in devices_long.split(|x| x.eq(&b'\n')) {
            if device.is_empty() {
                break;
            }

            vec_devices.push(DeviceLong::try_from(device.to_vec())?);
        }

        Ok(vec_devices)
    }

    /// Get a device, assuming that only this device is connected.
    pub fn get_device(&mut self) -> Result<ADBServerDevice> {
        let mut devices = self.devices()?.into_iter();
        match devices.next() {
            Some(device) => match devices.next() {
                Some(_) => Err(RustADBError::DeviceNotFound(
                    "too many devices connected".to_string(),
                )),
                None => Ok(ADBServerDevice::new(device.identifier, self.socket_addr)),
            },
            None => Err(RustADBError::DeviceNotFound(
                "no device connected".to_string(),
            )),
        }
    }

    /// Get a device matching the given name, if existing.
    /// - There is no device connected => Error
    /// - There is a single device connected => Ok
    /// - There are more than 1 device connected => Error
    pub fn get_device_by_name(&mut self, name: String) -> Result<ADBServerDevice> {
        let nb_devices = self
            .devices()?
            .into_iter()
            .filter(|d| d.identifier.as_str() == name)
            .collect::<Vec<DeviceShort>>()
            .len();
        if nb_devices != 1 {
            Err(RustADBError::DeviceNotFound(format!(
                "could not find device {name}"
            )))
        } else {
            Ok(ADBServerDevice::new(name.to_string(), self.socket_addr))
        }
    }

    /// Tracks new devices showing up.
    // TODO: Change with Generator when feature stabilizes
    pub fn track_devices(&mut self, callback: impl Fn(DeviceShort) -> Result<()>) -> Result<()> {
        self.connect()?.send_adb_request(AdbCommand::TrackDevices)?;

        loop {
            let length = self.get_transport()?.get_hex_body_length()?;

            if length > 0 {
                let mut body = vec![
                    0;
                    length
                        .try_into()
                        .map_err(|_| RustADBError::ConversionError)?
                ];
                self.get_transport()?
                    .get_connection()?
                    .read_exact(&mut body)?;

                callback(DeviceShort::try_from(body)?)?;
            }
        }
    }
}
