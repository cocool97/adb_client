use std::net::TcpStream;

use crate::server_device::ADBServerDevice;
use crate::{Result, RustADBError};

/// A sync session with a device, running any number of file operations on a single server connection.
///
/// Opened with [`ADBServerDevice::sync_session`]. While it exists, the device connection is dedicated to it.
/// The session ends with [`ADBSyncSession::quit`], or when dropped.
///
/// If a request fails (e.g. pulling a missing file), adbd ends the session: every later request returns an
/// error, and a new session must be opened.
///
/// ```rust no_run
/// use adb_client::{ADBListItemType, server::ADBServer};
/// use std::fs::File;
///
/// let mut server = ADBServer::default();
/// let mut device = server.get_device().expect("cannot get device");
///
/// // Pull all files of a directory, using a single connection
/// let mut sync = device.sync_session().expect("cannot open sync session");
/// for item in sync.list("/sdcard/Download").expect("cannot list directory") {
///     if let ADBListItemType::File(file) = item {
///         let mut output = File::create(&file.name).expect("cannot create file");
///         sync.pull(&format!("/sdcard/Download/{}", file.name), &mut output)
///             .expect("cannot pull file");
///     }
/// }
/// sync.quit().expect("cannot end sync session");
/// ```
#[derive(Debug)]
pub struct ADBSyncSession<'a> {
    pub(super) device: &'a mut ADBServerDevice,
    /// No more requests can be sent: session has been ended, or a previous request failed.
    pub(super) session_closed: bool,
}

impl ADBSyncSession<'_> {
    /// Runs a request on this session's connection.
    pub(super) fn run<T>(&mut self, request: impl FnOnce(&TcpStream) -> Result<T>) -> Result<T> {
        if self.session_closed {
            return Err(RustADBError::ADBRequestFailed(
                "sync session cannot be used anymore: it has been ended, or a previous request failed"
                    .into(),
            ));
        }

        let result = request(self.device.transport.get_raw_connection()?);

        // adbd ends the sync service after a failure, and an interrupted transfer leaves unread data on the
        // connection: no request can follow.
        if result.is_err() {
            self.session_closed = true;
        }

        result
    }
}

impl Drop for ADBSyncSession<'_> {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.send_quit();
    }
}
