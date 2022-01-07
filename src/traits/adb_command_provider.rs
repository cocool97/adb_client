use crate::{
    models::{AdbVersion, DeviceLong},
    Device, Result,
};

/// Represents the property to serve ADB commands.
pub trait AdbCommandProvider {
    /// Gets server's internal version number.
    fn version(&self) -> Result<AdbVersion>;
    /// Gets a list of connected devices.
    fn devices(&self) -> Result<Vec<Device>>;
    /// Gets an extended list of connected devices including the device paths in the state.
    fn devices_long(&self) -> Result<Vec<DeviceLong>>;
    /// Asks the ADB server to quit immediately.
    fn kill(&self) -> Result<()>;
    /// Tracks new devices showing up.
    fn track_devices(&self, callback: fn(Device) -> Result<()>) -> Result<()>;
}
