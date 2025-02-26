use std::str::FromStr;
use std::sync::LazyLock;
use std::{fmt::Display, str};

use crate::{DeviceState, RustADBError};
use regex::bytes::Regex;

static DEVICES_LONG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<identifier>\S+)\s+(?P<state>\w+)\s+(usb:(?P<usb1>\S+)|(?P<usb2>\S+))?\s*(product:(?P<product>\w+)\s+model:(?P<model>\w+)\s+device:(?P<device>\w+)\s+)?transport_id:(?P<transport_id>\d+)$").expect("cannot build devices long regex")
});

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
            usb: match groups.name("usb1") {
                None => match groups.name("usb2") {
                    None => "Unk".to_string(),
                    Some(usb) => String::from_utf8(usb.as_bytes().to_vec())?,
                },
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
