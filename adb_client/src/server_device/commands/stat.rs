use crate::{AdbStatResponse, Result, server_device::ADBServerDevice};

impl ADBServerDevice {
    /// Stat file given as path on the device.
    ///
    /// Opens a dedicated connection: use [`ADBServerDevice::sync_session`] to run several file operations on a
    /// single one.
    pub fn stat<A: AsRef<str>>(&self, path: A) -> Result<AdbStatResponse> {
        self.sync_session()?.stat(path.as_ref())
    }
}
