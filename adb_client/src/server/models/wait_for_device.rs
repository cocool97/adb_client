use std::fmt::Display;

use crate::RustADBError;

#[derive(Clone, Debug)]
/// List of available transports to wait for.
pub enum WaitForDeviceTransport {
    /// USB transport
    Usb,
    /// Local transport
    Local,
    /// Any transport (default value)
    Any,
}

impl Default for WaitForDeviceTransport {
    fn default() -> Self {
        Self::Any
    }
}

impl Display for WaitForDeviceTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaitForDeviceTransport::Usb => write!(f, "usb"),
            WaitForDeviceTransport::Local => write!(f, "local"),
            WaitForDeviceTransport::Any => write!(f, "any"),
        }
    }
}

impl TryFrom<&str> for WaitForDeviceTransport {
    type Error = RustADBError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "usb" => Ok(Self::Usb),
            "local" => Ok(Self::Local),
            "any" => Ok(Self::Any),
            t => Err(RustADBError::UnknownTransport(t.to_string())),
        }
    }
}

#[derive(Debug)]
/// List of available states to wait for.
pub enum WaitForDeviceState {
    /// Device in "device" state
    Device,
    /// Device in "recovery" state
    Recovery,
    /// Device in "sideload" state
    Sideload,
    /// Device in "bootloader" state
    Bootloader,
}

impl Display for WaitForDeviceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaitForDeviceState::Device => write!(f, "device"),
            WaitForDeviceState::Recovery => write!(f, "recovery"),
            WaitForDeviceState::Sideload => write!(f, "sideload"),
            WaitForDeviceState::Bootloader => write!(f, "bootloader"),
        }
    }
}
