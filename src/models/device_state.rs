use std::{fmt::Display, str::FromStr};

use crate::RustADBError;

/// Represents the connection state of the device.
#[derive(Debug)]
pub enum DeviceState {
    /// The device is not connected to adb or is not responding.
    Offline,
    /// The device is now connected to the adb server. Note that this state does not imply that the Android system is fully booted and operational because the device connects to adb while the system is still booting. However, after boot-up, this is the normal operational state of an device.
    Device,
    /// There is no device connected.
    NoDevice,
    /// Device is being authorized
    Authorizing,
    /// The device is unauthorized.
    Unauthorized,
}

impl Display for DeviceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceState::Offline => write!(f, "offline"),
            DeviceState::Device => write!(f, "device"),
            DeviceState::NoDevice => write!(f, "no device"),
            DeviceState::Authorizing => write!(f, "authorizing"),
            DeviceState::Unauthorized => write!(f, "unauthorized"),
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
            "authorizing" => Ok(Self::Authorizing),
            "unauthorized" => Ok(Self::Unauthorized),
            _ => Err(RustADBError::UnknownDeviceState(lowercased)),
        }
    }
}
