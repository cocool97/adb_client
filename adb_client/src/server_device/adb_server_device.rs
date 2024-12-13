use crate::{ADBTransport, Result, TCPServerTransport};
use std::net::SocketAddrV4;

/// Represents a device connected to the ADB server.
#[derive(Debug)]
pub struct ADBServerDevice {
    /// Unique device identifier.
    pub identifier: String,
    /// Internal [TCPServerTransport]
    pub(crate) transport: TCPServerTransport,
}

impl ADBServerDevice {
    /// Instantiates a new [ADBServerDevice]
    pub fn new(identifier: String, socket_addr: Option<SocketAddrV4>) -> Self {
        let transport = if let Some(addr) = socket_addr {
            TCPServerTransport::new(addr)
        } else {
            TCPServerTransport::default()
        };

        Self {
            identifier,
            transport,
        }
    }

    /// Connect to underlying transport
    pub(crate) fn connect(&mut self) -> Result<&mut TCPServerTransport> {
        self.transport.connect()?;

        Ok(&mut self.transport)
    }
}

impl Drop for ADBServerDevice {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.transport.disconnect();
    }
}
