use std::io::{Result, Write};

use crate::message_devices::{
    adb_message_transport::ADBMessageTransport, adb_session::ADBSession,
    adb_transport_message::ADBTransportMessage, message_commands::MessageCommand,
};

/// [`Write`] trait implementation to hide underlying ADB protocol write logic.
///
/// Read received responses to check that message has been correctly received.
pub struct MessageWriter<T: ADBMessageTransport> {
    session: ADBSession<T>,
}

impl<T: ADBMessageTransport> MessageWriter<T> {
    pub fn new(session: ADBSession<T>) -> Self {
        Self { session }
    }
}

impl<T: ADBMessageTransport> Write for MessageWriter<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let message = ADBTransportMessage::try_new(
            MessageCommand::Write,
            self.session.local_id(),
            self.session.remote_id(),
            buf,
        )
        .map_err(std::io::Error::other)?;

        self.session
            .send_and_expect_okay(message)
            .map_err(std::io::Error::other)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
