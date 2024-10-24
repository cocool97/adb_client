use std::io::Write;

use crate::USBTransport;

use super::{ADBUsbMessage, USBCommand};

/// Wraps a `Writer` to hide underlying ADB protocol write logic.
pub struct USBShellWriter {
    transport: USBTransport,
    local_id: u32,
    remote_id: u32,
}

impl USBShellWriter {
    pub fn new(transport: USBTransport, local_id: u32, remote_id: u32) -> Self {
        Self {
            transport,
            local_id,
            remote_id,
        }
    }
}

impl Write for USBShellWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let message = ADBUsbMessage::new(
            USBCommand::Write,
            self.local_id,
            self.remote_id,
            buf.to_vec(),
        );
        self.transport
            .write_message(message)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
