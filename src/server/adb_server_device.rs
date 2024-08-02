use std::net::SocketAddrV4;

use crate::{ADBTransport, Result, RustADBError, TCPServerProtocol};

/// Represents a device connected to the ADB server.
#[derive(Debug)]
pub struct ADBServerDevice {
    /// Unique device identifier.
    pub identifier: String,
    /// Address to connect to
    pub(crate) socket_addr: Option<SocketAddrV4>,
    /// Internal [TcpStream], lazily initialized
    pub(crate) transport: Option<TCPServerProtocol>,
}

impl ADBServerDevice {
    /// Instantiates a new [ADBServerDevice]
    pub fn new(identifier: String, socket_addr: Option<SocketAddrV4>) -> Self {
        Self {
            identifier,
            transport: None,
            socket_addr,
        }
    }

    pub(crate) fn get_transport(&mut self) -> Result<&mut TCPServerProtocol> {
        self.transport
            .as_mut()
            .ok_or(RustADBError::IOError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )))
    }

    /// Connect to underlying transport
    pub(crate) fn connect(&mut self) -> Result<&mut TCPServerProtocol> {
        let mut transport = if let Some(addr) = &self.socket_addr {
            TCPServerProtocol::new(*addr)
        } else {
            TCPServerProtocol::default()
        };
        transport.connect()?;
        self.transport = Some(transport);

        self.get_transport()
    }
}

impl Drop for ADBServerDevice {
    fn drop(&mut self) {
        if let Some(ref mut transport) = &mut self.transport {
            let _ = transport.disconnect();
        }
    }
}
