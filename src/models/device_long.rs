use lazy_static::lazy_static;
use std::str::FromStr;
use std::{fmt::Display, str};

use regex::bytes::Regex;

use crate::{DeviceState, RustADBError};

lazy_static! {
    static ref DEVICES_LONG_REGEX: Regex = Regex::new("^(?P<identifier>\\S+)\\s+(?P<state>\\w+) (usb:(?P<usb>.*) )?(product:(?P<product>\\w+) model:(?P<model>\\w+) device:(?P<device>\\w+) )?transport_id:(?P<transport_id>\\d+)$").unwrap();
}

/// Represents a new device with more informations helded.
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

impl TryFrom<Vec<u8>> for DeviceLong {
    type Error = RustADBError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {

        let groups = DEVICES_LONG_REGEX
            .captures(&value)
            .ok_or(RustADBError::RegexParsingError)?;

        Ok(DeviceLong {
            identifier: String::from_utf8(
                groups
                    .name("identifier")
                    .ok_or(RustADBError::RegexParsingError)?
                    .as_bytes()
                    .to_vec(),
            )?,
            state: DeviceState::from_str(&String::from_utf8(
                groups
                    .name("state")
                    .ok_or(RustADBError::RegexParsingError)?
                    .as_bytes()
                    .to_vec(),
            )?)?,
            usb: match groups.name("usb") {
                None => "Unk".to_string(),
                Some(usb) => String::from_utf8(usb.as_bytes().to_vec())?,
            },
            product: match groups.name("product") {
                None => "Unk".to_string(),
                Some(product) => String::from_utf8(product.as_bytes().to_vec())?,
            },
            model: match groups.name("model") {
                None => "Unk".to_string(),
                Some(model) => String::from_utf8(model.as_bytes().to_vec())?,
            },
            device: match groups.name("device") {
                None => "Unk".to_string(),
                Some(device) => String::from_utf8(device.as_bytes().to_vec())?,
            },
            transport_id: u32::from_str_radix(
                str::from_utf8(
                    groups
                        .name("transport_id")
                        .ok_or(RustADBError::RegexParsingError)?
                        .as_bytes(),
                )?,
                16,
            )?,
        })
    }
}
