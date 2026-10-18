use std::io::Write;

use crate::{Result, server_device::ADBServerDevice};

impl ADBServerDevice {
    /// Receives path to stream from the device.
    ///
    /// Opens a dedicated connection: use [`ADBServerDevice::sync_session`] to run several file operations on a
    /// single one.
    pub fn pull(&mut self, path: &dyn AsRef<str>, stream: &mut dyn Write) -> Result<()> {
        self.sync_session()?.pull(path.as_ref(), stream)
    }
}
