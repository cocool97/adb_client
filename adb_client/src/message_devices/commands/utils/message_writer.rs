use std::io::{Error, ErrorKind, Result, Write};

use crate::message_devices::{
    adb_message_transport::ADBMessageTransport, adb_transport_message::ADBTransportMessage,
    message_commands::MessageCommand,
};

/// [`Write`] trait implementation to hide underlying ADB protocol write logic.
///
/// Read received responses to check that message has been correctly received.
pub struct MessageWriter<T: ADBMessageTransport> {
    transport: T,
    local_id: u32,
    remote_id: u32,
}

impl<T: ADBMessageTransport> MessageWriter<T> {
    pub fn new(transport: T, local_id: u32, remote_id: u32) -> Self {
        Self {
            transport,
            local_id,
            remote_id,
        }
    }
}

impl<T: ADBMessageTransport> Write for MessageWriter<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let message =
            ADBTransportMessage::new(MessageCommand::Write, self.local_id, self.remote_id, buf);
        self.transport
            .write_message(message)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        match self.transport.read_message() {
            Ok(response) => {
                response
                    .assert_command(MessageCommand::Okay)
                    .map_err(Error::other)?;
                Ok(buf.len())
            }
            Err(e) => Err(Error::other(e)),
        }
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
