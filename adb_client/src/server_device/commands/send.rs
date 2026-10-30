use std::io::Read;

use crate::{Result, server_device::ADBServerDevice};

impl ADBServerDevice {
    /// Send stream to path on the device.
    ///
    /// Opens a dedicated connection: use [`ADBServerDevice::sync_session`] to run several file operations on a
    /// single one.
    pub fn push<R: Read, A: AsRef<str>>(&self, mut stream: R, path: A) -> Result<()> {
        log::info!("Sending data to {}", path.as_ref());
        self.sync_session()?.push(&mut stream, path.as_ref())
    }
}
