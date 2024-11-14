use std::io::{ErrorKind, Write};

use crate::USBTransport;

use super::{ADBUsbMessage, USBCommand};

/// Wraps a `Writer` to hide underlying ADB protocol write logic.
///
/// Read received responses to check that message has been received.
pub struct USBWriter {
    transport: USBTransport,
    local_id: u32,
    remote_id: u32,
}

impl USBWriter {
    pub fn new(transport: USBTransport, local_id: u32, remote_id: u32) -> Self {
        Self {
            transport,
            local_id,
            remote_id,
        }
    }
}

impl Write for USBWriter {
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

        match self.transport.read_message() {
            Ok(response) => match response.header().command() {
                USBCommand::Okay => Ok(buf.len()),
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
