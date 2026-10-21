use std::{
    ops::{Deref, DerefMut},
    sync::MutexGuard,
};

use adb_client::ADBDeviceExt;

use crate::utils::adb_file::RemoteFile;

/// Device under test, removing the files created through [`TestDevice::remote_file`] when dropped.
///
/// Cleanup goes through the device itself: some transports (USB) cannot be opened twice at the same time.
pub(crate) struct TestDevice<D: ADBDeviceExt> {
    // Dropped before `_lock`, so the device is released before another test can open it
    device: D,
    remote_files: Vec<String>,
    _lock: Option<MutexGuard<'static, ()>>,
}

impl<D: ADBDeviceExt> TestDevice<D> {
    pub(crate) const fn new(device: D) -> Self {
        Self {
            device,
            remote_files: Vec::new(),
            _lock: None,
        }
    }

    /// Device that must not be used by several tests at once, `lock` being held as long as it lives.
    pub(crate) const fn exclusive(device: D, lock: MutexGuard<'static, ()>) -> Self {
        Self {
            device,
            remote_files: Vec::new(),
            _lock: Some(lock),
        }
    }

    /// Returns a test file path, removed from the device once this one is dropped.
    pub(crate) fn remote_file(&mut self, name: &str) -> RemoteFile {
        let remote = RemoteFile::new(name);
        self.remote_files.push(remote.as_ref().to_string());
        remote
    }
}

impl<D: ADBDeviceExt> Deref for TestDevice<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl<D: ADBDeviceExt> DerefMut for TestDevice<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.device
    }
}

impl<D: ADBDeviceExt> Drop for TestDevice<D> {
    fn drop(&mut self) {
        if !self.remote_files.is_empty() {
            let command = format!("rm -f {}", self.remote_files.join(" "));
            let _ = self.device.shell_command(&command, None, None);
        }
    }
}
