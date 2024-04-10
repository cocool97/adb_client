use lazy_static::lazy_static;
use regex::bytes::Regex;
use std::{fmt::Display, str::FromStr};

use crate::{DeviceState, RustADBError};

lazy_static! {
    static ref DEVICES_REGEX: Regex = Regex::new("^(\\S+)\t(\\w+)\n?$").unwrap();
}

/// Represents a device connected to the ADB server.
#[derive(Debug)]
pub struct Device {
    /// Unique device identifier.
    pub identifier: String,
    /// Connection state of the device.
    pub state: DeviceState,
}

impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.identifier, self.state)
    }
}

impl TryFrom<Vec<u8>> for Device {
    type Error = RustADBError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // Optional final '\n' is used to match TrackDevices inputs
        let groups = DEVICES_REGEX.captures(&value).unwrap();
        Ok(Device {
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
        })
    }
}
