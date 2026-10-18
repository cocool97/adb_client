use adb_client::ADBDeviceExt;

use crate::utils::get_connected_device;

/// Directory writable by the `shell` user on every Android version.
pub(crate) const REMOTE_DIR: &str = "/data/local/tmp";

/// Path of a test file on the device, removed when dropped (even if the test fails).
pub(crate) struct RemoteFile(String);

impl RemoteFile {
    pub(crate) fn new(name: &str) -> Self {
        Self(format!(
            "{REMOTE_DIR}/adb_client_test_{}_{name}",
            std::process::id()
        ))
    }

    pub(crate) fn file_name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }
}

impl AsRef<str> for RemoteFile {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Drop for RemoteFile {
    fn drop(&mut self) {
        let mut device = get_connected_device();
        let _ = device.shell_command(&format!("rm -f {}", self.0), None, None);
    }
}
