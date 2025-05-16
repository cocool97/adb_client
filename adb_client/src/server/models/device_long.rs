use std::str::FromStr;
use std::{fmt::Display, str};

use crate::{DeviceState, RustADBError};

/// Represents a new device with more informations.
#[derive(Debug)]
pub struct DeviceLong {
    /// Unique device identifier.
    pub identifier: String,
    /// Connection state of the device.
    pub state: DeviceState,
    /// Usb port used by the device.
    pub usb: String,
    /// Product code.
    pub product: String,
    /// Device model.
    pub model: String,
    /// Device code.
    pub device: String,
    /// Transport identifier.
    pub transport_id: u32,
}

impl Display for DeviceLong {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}       {} usb:{} product:{} model:{} device:{} transport_id:{}",
            self.identifier,
            self.state,
            self.usb,
            self.product,
            self.model,
            self.device,
            self.transport_id
        )
    }
}

impl TryFrom<&[u8]> for DeviceLong {
    type Error = RustADBError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let value = str::from_utf8(value)?;
        let mut it = value.split_whitespace();

        let id = it.next().ok_or(Self::Error::RegexParsingError)?;
        let stat = DeviceState::from_str(
            it.next()
                .ok_or(Self::Error::UnknownDeviceState(String::new()))?,
        )?;

        let mut comp = it.next().ok_or(Self::Error::RegexParsingError)?;

        let mut usb = match comp.strip_prefix("usb:") {
            Some(usb) => {
                comp = it.next().ok_or(Self::Error::RegexParsingError)?;
                Some(usb)
            }
            _ => None,
        };

        let prod = match comp.strip_prefix("product:") {
            Some(prod) => {
                comp = it.next().ok_or(Self::Error::RegexParsingError)?;
                Some(prod)
            }
            _ => {
                usb = Some(comp);
                comp = it.next().ok_or(Self::Error::RegexParsingError)?;
                comp.strip_prefix("product:")
            }
        };
        if prod.is_some() {
            comp = it.next().ok_or(Self::Error::RegexParsingError)?
        }

        let model = comp.strip_prefix("model:");
        if model.is_some() {
            comp = it.next().ok_or(Self::Error::RegexParsingError)?
        }

        let dev = comp.strip_prefix("device:");
        if dev.is_some() {
            comp = it.next().ok_or(Self::Error::RegexParsingError)?
        }

        if it.next().is_some() {
            return Err(Self::Error::RegexParsingError);
        }

        let trans = comp
            .strip_prefix("transport_id:")
            .ok_or(Self::Error::UnknownTransport(String::new()))?;

        Ok(DeviceLong {
            identifier: id.to_string(),
            state: stat,
            usb: usb.unwrap_or("Unk").to_string(),
            product: prod.unwrap_or("Unk").to_string(),
            model: model.unwrap_or("Unk").to_string(),
            device: dev.unwrap_or("Unk").to_string(),
            transport_id: trans
                .parse()
                .map_err(|_| Self::Error::UnknownTransport(trans.to_string()))?,
        })
    }
}

#[test]
fn test_static_devices_long() {
    let inputs = [
        "7a5158f05122195aa       device 1-5 product:gts210vewifixx model:SM_T813 device:gts210vewifi transport_id:4",
        "n311r05e               device usb:0-1.5 product:alioth model:M2012K11AC device:alioth transport_id:58",
        "192.168.100.192:5555   device product:alioth model:M2012K11AC device:alioth transport_id:97",
        "emulator-5554          device product:sdk_gphone64_arm64 model:sdk_gphone64_arm64 device:emu64a transport_id:101",
        "QQ20131020250511       device 20-4 product:NOH-AN00 model:NOH_AN00 device:HWNOH transport_id:3",
    ];
    for input in inputs {
        DeviceLong::try_from(input.as_bytes()).expect(&format!("cannot parse input: '{input}'"));
    }
}
