use std::io::{ErrorKind, Write};

use crate::ADBMessageTransport;

use super::{ADBTransportMessage, MessageCommand};

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
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let message =
            ADBTransportMessage::new(MessageCommand::Write, self.local_id, self.remote_id, buf);
        self.transport
            .write_message(message)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        match self.transport.read_message() {
            Ok(response) => match response.header().command() {
                MessageCommand::Okay => Ok(buf.len()),
                c => Err(std::io::Error::new(
                    ErrorKind::Other,
                    format!("wrong response received: {c}"),
                )),
            },
            Err(e) => Err(std::io::Error::new(ErrorKind::Other, e)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
