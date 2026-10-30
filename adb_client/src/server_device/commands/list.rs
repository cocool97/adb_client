use crate::{ADBListItemType, Result, server_device::ADBServerDevice};

impl ADBServerDevice {
    /// Lists files in path on the device.
    /// note: path uses internal file paths, so Documents is at /storage/emulated/0/Documents
    ///
    /// Opens a dedicated connection: use [`ADBServerDevice::sync_session`] to run several file operations on a
    /// single one.
    pub fn list<A: AsRef<str>>(&self, path: A) -> Result<Vec<ADBListItemType>> {
        self.sync_session()?.list(path.as_ref())
    }
}
