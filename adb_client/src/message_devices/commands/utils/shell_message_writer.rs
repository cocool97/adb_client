use std::{
    io::{ErrorKind, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::message_devices::{
    adb_message_transport::ADBMessageTransport, adb_transport_message::ADBTransportMessage,
    message_commands::MessageCommand,
};

/// [`Write`] trait implementation to hide underlying ADB protocol write logic for shell commands.
pub struct ShellMessageWriter<T: ADBMessageTransport> {
    transport: T,
    local_id: u32,
    remote_id: u32,
    /// Set once the stream has been closed by either side.
    stream_closed: Arc<AtomicBool>,
}

impl<T: ADBMessageTransport> ShellMessageWriter<T> {
    pub const fn new(
        transport: T,
        local_id: u32,
        remote_id: u32,
        stream_closed: Arc<AtomicBool>,
    ) -> Self {
        Self {
            transport,
            local_id,
            remote_id,
            stream_closed,
        }
    }

    /// Close the stream from our side, unless it has already been closed.
    pub fn close(&mut self) -> crate::Result<()> {
        if self.stream_closed.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        self.transport.write_message(ADBTransportMessage::try_new(
            MessageCommand::Clse,
            self.local_id,
            self.remote_id,
            &[],
        )?)
    }
}

impl<T: ADBMessageTransport> Write for ShellMessageWriter<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.stream_closed.load(Ordering::SeqCst) {
            return Err(std::io::Error::new(
                ErrorKind::BrokenPipe,
                "stream closed by remote",
            ));
        }

        let message =
            ADBTransportMessage::try_new(MessageCommand::Write, self.local_id, self.remote_id, buf)
                .map_err(std::io::Error::other)?;
        self.transport
            .write_message(message)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
