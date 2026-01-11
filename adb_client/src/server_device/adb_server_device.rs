use crate::{
    Result,
    adb_transport::{ADBConnectableTransport, ADBDisconnectableTransport, Connected, NotConnected},
    models::{ADBCommand, ADBHostCommand},
    server::TCPServerTransport,
};
use std::{marker::PhantomData, net::SocketAddrV4};

/// Represents a device connected to the ADB server.
#[derive(Debug)]
pub struct ADBServerDevice<T> {
    inner_state: PhantomData<T>,
    /// Unique device identifier.
    pub identifier: Option<String>,
    /// Internal [`TCPServerTransport`]
    pub(crate) transport: TCPServerTransport,
}

impl ADBServerDevice<NotConnected> {
    /// Instantiates a new [`ADBServerDevice`], knowing its ADB identifier (as returned by `adb devices` command).
    pub fn new(
        identifier: String,
        server_addr: Option<SocketAddrV4>,
    ) -> Result<ADBServerDevice<Connected>> {
        Self::new_inner(Some(identifier), server_addr)
    }

    /// Instantiates a new [`ADBServerDevice`], assuming only one is currently connected.
    pub fn autodetect(server_addr: Option<SocketAddrV4>) -> Result<ADBServerDevice<Connected>> {
        Self::new_inner(None, server_addr)
    }

    fn new_inner(
        identifier: Option<String>,
        server_addr: Option<SocketAddrV4>,
    ) -> Result<ADBServerDevice<Connected>> {
        let mut transport = TCPServerTransport::new_or_default(server_addr);

        transport.connect()?;

        Ok(ADBServerDevice::<Connected> {
            inner_state: PhantomData::<Connected>,
            identifier,
            transport,
        })
    }
}

impl ADBServerDevice<Connected> {
    /// Set device connection to use serial transport
    pub(crate) fn set_serial_transport(&mut self) -> Result<()> {
        let identifier = self.identifier.clone();
        if let Some(serial) = identifier {
            self.transport
                .send_adb_request(&ADBCommand::Host(ADBHostCommand::TransportSerial(serial)))?;
        } else {
            self.transport
                .send_adb_request(&ADBCommand::Host(ADBHostCommand::TransportAny))?;
        }

        Ok(())
    }
}

impl<T> Drop for ADBServerDevice<T> {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.transport.disconnect();
    }
}
