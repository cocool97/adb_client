use super::{ADBSyncSession, send_sync_request};
use crate::{Result, models::SyncCommand};

impl ADBSyncSession<'_> {
    /// Ends this session, telling the device that no more requests will follow.
    pub fn quit(mut self) -> Result<()> {
        self.send_quit()
    }

    /// Sends `QUIT`, unless the session is already closed.
    pub(super) fn send_quit(&mut self) -> Result<()> {
        if self.session_closed {
            return Ok(());
        }
        self.session_closed = true;

        send_sync_request(
            self.device.transport.get_raw_connection()?,
            &SyncCommand::Quit,
            &[],
        )
    }
}
