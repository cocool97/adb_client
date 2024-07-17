use crate::Result;
use crate::RustADBError;
use crate::TCPServerProtocol;
use crate::Transport;
use std::net::SocketAddrV4;

/// Represents an ADB Server
#[derive(Debug, Default)]
pub struct ADBServer {
    /// Internal [TcpStream], lazily initialized
    pub(crate) transport: Option<TCPServerProtocol>,
    /// Address to connect to
    pub(crate) socket_addr: Option<SocketAddrV4>,
}

impl ADBServer {
    /// Instantiates a new [ADBServer]
    pub fn new(address: SocketAddrV4) -> Self {
        Self {
            transport: None,
            socket_addr: Some(address),
        }
    }

    /// Returns the current selected transport
    pub(crate) fn get_transport(&mut self) -> Result<&mut TCPServerProtocol> {
        self.transport
            .as_mut()
            .ok_or(RustADBError::IOError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "server connection not initialized",
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

impl Drop for ADBServer {
    fn drop(&mut self) {
        if let Some(ref mut transport) = &mut self.transport {
            let _ = transport.disconnect();
        }
    }
}
