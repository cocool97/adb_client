use crate::{Device, Result};

/// Represents the property to serve ADB commands.
pub trait AdbCommandProvider {
    /// Gets server's internal version number.
    fn version(&self) -> Result<()>;
    /// Gets a list of connected devices.
    fn devices(&self) -> Result<Vec<Device>>;
    /// Gets an extended list of connected devices including the device paths in the state.
    fn devices_long(&self) -> Result<Vec<Device>>;
}
