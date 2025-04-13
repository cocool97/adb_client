use std::io::Write;

use crate::ADBMessageTransport;

use super::{ADBTransportMessage, models::MessageCommand};

/// [`Write`] trait implementation to hide underlying ADB protocol write logic for shell commands.
pub struct ShellMessageWriter<T: ADBMessageTransport> {
    transport: T,
    local_id: u32,
    remote_id: u32,
}

impl<T: ADBMessageTransport> ShellMessageWriter<T> {
    pub fn new(transport: T, local_id: u32, remote_id: u32) -> Self {
        Self {
            transport,
            local_id,
            remote_id,
        }
    }
}

impl<T: ADBMessageTransport> Write for ShellMessageWriter<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let message =
            ADBTransportMessage::new(MessageCommand::Write, self.local_id, self.remote_id, buf);
        self.transport
            .write_message(message)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
