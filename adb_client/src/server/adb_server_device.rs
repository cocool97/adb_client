use crate::{ADBTransport, Result, TCPServerTransport};
use std::net::SocketAddrV4;

/// Represents a device connected to the ADB server.
#[derive(Debug)]
pub struct ADBServerDevice {
    /// Unique device identifier.
    pub identifier: String,
    /// Internal [TCPServerTransport]
    transport: TCPServerTransport,
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

    pub(crate) fn get_transport(&self) -> &TCPServerTransport {
        &self.transport
    }

    pub(crate) fn get_transport_mut(&mut self) -> &mut TCPServerTransport {
        &mut self.transport
    }

    /// Connect to underlying transport
    pub(crate) fn connect(&mut self) -> Result<&mut TCPServerTransport> {
        self.transport.connect()?;

        Ok(self.get_transport_mut())
    }
}

impl Drop for ADBServerDevice {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.transport.disconnect();
    }
}
