use std::str::FromStr;
use std::{fmt::Display, str};

use regex::bytes::Regex;

use crate::{DeviceState, RustADBError};

pub struct DeviceLong {
    identifier: String,
    state: DeviceState,
    usb: String,
    product: String,
    model: String,
    device: String,
    transport_id: u32,
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

    // TODO: Prevent regex compilation every call to try_from()
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let parse_regex = Regex::new("^(\\w+)       (\\w+) usb:(.*) product:(\\w+) model:(\\w+) device:(\\w+) transport_id:(\\d+)$")?;

        let groups = parse_regex.captures(&value).unwrap();
        Ok(DeviceLong {
            identifier: String::from_utf8(
                groups
                    .get(1)
                    .ok_or(RustADBError::RegexParsingError)?
                    .as_bytes()
                    .to_vec(),
            )?,
            state: DeviceState::from_str(&String::from_utf8(
                groups
                    .get(2)
                    .ok_or(RustADBError::RegexParsingError)?
                    .as_bytes()
                    .to_vec(),
            )?)?,
            usb: String::from_utf8(
                groups
                    .get(3)
                    .ok_or(RustADBError::RegexParsingError)?
                    .as_bytes()
                    .to_vec(),
            )?,
            product: String::from_utf8(
                groups
                    .get(4)
                    .ok_or(RustADBError::RegexParsingError)?
                    .as_bytes()
                    .to_vec(),
            )?,
            model: String::from_utf8(
                groups
                    .get(5)
                    .ok_or(RustADBError::RegexParsingError)?
                    .as_bytes()
                    .to_vec(),
            )?,
            device: String::from_utf8(
                groups
                    .get(6)
                    .ok_or(RustADBError::RegexParsingError)?
                    .as_bytes()
                    .to_vec(),
            )?,
            transport_id: u32::from_str_radix(
                str::from_utf8(
                    groups
                        .get(7)
                        .ok_or(RustADBError::RegexParsingError)?
                        .as_bytes(),
                )?,
                16,
            )?,
        })
    }
}
