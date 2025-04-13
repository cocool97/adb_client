use crate::{ADBTransport, Result, TCPServerTransport, models::AdbServerCommand};
use std::net::SocketAddrV4;

/// Represents a device connected to the ADB server.
#[derive(Debug)]
pub struct ADBServerDevice {
    /// Unique device identifier.
    pub identifier: Option<String>,
    /// Internal [TCPServerTransport]
    pub(crate) transport: TCPServerTransport,
}

impl ADBServerDevice {
    /// Instantiates a new [ADBServerDevice], knowing its ADB identifier (as returned by `adb devices` command).
    pub fn new(identifier: String, server_addr: Option<SocketAddrV4>) -> Self {
        let transport = TCPServerTransport::new_or_default(server_addr);

        Self {
            identifier: Some(identifier),
            transport,
        }
    }

    /// Instantiates a new [ADBServerDevice], assuming only one is currently connected.
    pub fn autodetect(server_addr: Option<SocketAddrV4>) -> Self {
        let transport = TCPServerTransport::new_or_default(server_addr);

        Self {
            identifier: None,
            transport,
        }
    }

    /// Connect to underlying transport
    pub(crate) fn connect(&mut self) -> Result<&mut TCPServerTransport> {
        self.transport.connect()?;

        Ok(&mut self.transport)
    }

    /// Set device connection to use serial transport
    pub(crate) fn set_serial_transport(&mut self) -> Result<()> {
        let identifier = self.identifier.clone();
        let transport = self.connect()?;
        if let Some(serial) = identifier {
            transport.send_adb_request(AdbServerCommand::TransportSerial(serial))?;
        } else {
            transport.send_adb_request(AdbServerCommand::TransportAny)?;
        }

        Ok(())
    }
}

impl Drop for ADBServerDevice {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.transport.disconnect();
    }
}
