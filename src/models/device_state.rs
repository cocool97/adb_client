use std::{fmt::Display, str::FromStr};

use crate::RustADBError;

pub enum DeviceState {
    Offline,
    Device,
    NoDevice,
}

impl Display for DeviceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceState::Offline => write!(f, "offline"),
            DeviceState::Device => write!(f, "device"),
            DeviceState::NoDevice => write!(f, "no device"),
        }
    }
}

impl FromStr for DeviceState {
    type Err = RustADBError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lowercased = s.to_ascii_lowercase();
        match lowercased.as_str() {
            "offline" => Ok(Self::Offline),
            "device" => Ok(Self::Device),
            "no device" => Ok(Self::NoDevice),
            _ => Err(RustADBError::UnknownDeviceState(lowercased)),
        }
    }
}
