use std::{fmt::Display, str::FromStr};

use regex::bytes::Regex;

use crate::{DeviceState, RustADBError};

/// Represents a device connected to the ADB server.
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

    // TODO: Prevent regex compilation every call to try_from()
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // Optional final '\n' is used to match TrackDevices inputs
        let parse_regex = Regex::new("^(\\w+)\t(\\w+)\n?$")?;

        let groups = parse_regex.captures(&value).unwrap();
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
