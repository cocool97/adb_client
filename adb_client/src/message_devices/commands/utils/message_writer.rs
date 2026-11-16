use std::io::{Result, Write};

use crate::message_devices::{adb_message_transport::ADBMessageTransport, adb_session::ADBSession};

/// [`Write`] trait implementation to hide underlying ADB protocol write logic.
///
/// Read received responses to check that message has been correctly received. Each write sends at most one
/// message, of the session maximum payload size.
pub struct MessageWriter<'session, T: ADBMessageTransport> {
    session: &'session mut ADBSession<T>,
}

impl<'session, T: ADBMessageTransport> MessageWriter<'session, T> {
    pub const fn new(session: &'session mut ADBSession<T>) -> Self {
        Self { session }
    }
}

impl<T: ADBMessageTransport> Write for MessageWriter<'_, T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let buf = &buf[..buf.len().min(self.session.max_payload_size())];
        self.session
            .send_write_and_expect_okay(buf)
            .map_err(std::io::Error::other)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
