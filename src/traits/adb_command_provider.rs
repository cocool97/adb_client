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
    /// Asks ADB server to switch the connection to either the device or emulator connect to/running on the host. Will fail if there is more than one such device/emulator available.
    fn transport_any(&self) -> Result<()>;
    /// Runs 'command' in a shell on the device, and return its output and error streams.
    fn shell_command(&self, serial: Option<String>, command: Vec<String>) -> Result<String>;
    /// Starts an interactive shell session on the device. Redirects stdin/stdout/stderr as appropriate.
    fn shell(&self, serial: Option<String>) -> Result<()>;
}
