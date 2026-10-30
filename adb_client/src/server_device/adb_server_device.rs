use crate::{
    ADBTransport, Result,
    models::{ADBCommand, ADBHostCommand, HostFeatures},
    server::TCPServerTransport,
};
use std::{net::SocketAddrV4, sync::Mutex};

/// Represents a device connected to the ADB server.
///
/// Each command opens its own connection to the server, so commands can run at the same time from several
/// threads.
#[derive(Debug)]
pub struct ADBServerDevice {
    /// Unique device identifier.
    pub identifier: Option<String>,
    /// Optional transport id assigned by the ADB server.
    ///
    /// When set, this takes precedence over `identifier` for transport selection.
    /// Required to disambiguate devices that report the same serial number.
    ///
    /// Note: transport ids are volatile. They are reassigned by the ADB server on
    /// device reconnect or server restart, so callers must re-query `devices_long`
    /// each time rather than caching the id across operations.
    pub transport_id: Option<u32>,
    /// Address of the ADB server
    server_addr: SocketAddrV4,
    /// Device features, fetched once
    pub(crate) cached_features: Mutex<Option<CachedFeatures>>,
}

/// Available features of a device
#[derive(Debug)]
pub struct CachedFeatures {
    pub(crate) identifier: Option<String>,
    pub(crate) transport_id: Option<u32>,
    pub(crate) features: Vec<HostFeatures>,
}

impl ADBServerDevice {
    /// Instantiates a new [`ADBServerDevice`], knowing its ADB identifier (as returned by `adb devices` command).
    #[must_use]
    pub fn new(identifier: String, server_addr: Option<SocketAddrV4>) -> Self {
        let server_addr = TCPServerTransport::new_or_default(server_addr).get_socketaddr();

        Self {
            identifier: Some(identifier),
            transport_id: None,
            server_addr,
            cached_features: Mutex::new(None),
        }
    }

    /// Instantiates a new [`ADBServerDevice`] selected by its transport id (as returned by `adb devices -l`).
    ///
    /// Use this when multiple devices share the same serial number, since transport id is
    /// always unique within a running ADB server. The id is volatile: do not cache it across
    /// device reconnects or server restarts — re-query via [`crate::server::ADBServer::devices_long`].
    #[must_use]
    pub fn new_with_transport_id(transport_id: u32, server_addr: Option<SocketAddrV4>) -> Self {
        let server_addr = TCPServerTransport::new_or_default(server_addr).get_socketaddr();

        Self {
            identifier: None,
            transport_id: Some(transport_id),
            server_addr,
            cached_features: Mutex::new(None),
        }
    }

    /// Instantiates a new [`ADBServerDevice`], assuming only one is currently connected.
    #[must_use]
    pub fn autodetect(server_addr: Option<SocketAddrV4>) -> Self {
        let server_addr = TCPServerTransport::new_or_default(server_addr).get_socketaddr();

        Self {
            identifier: None,
            transport_id: None,
            server_addr,
            cached_features: Mutex::new(None),
        }
    }

    /// Address of the ADB server
    pub(crate) const fn server_addr(&self) -> SocketAddrV4 {
        self.server_addr
    }

    /// Forget cached device features, they will be fetched again when needed.
    ///
    /// Must be called after commands restarting adbd, as it may then advertise different features.
    pub(crate) fn invalidate_features(&self) {
        if let Ok(mut cached_features) = self.cached_features.lock() {
            *cached_features = None;
        }
    }

    /// Open a new connection to the server, without selecting a device.
    pub(crate) fn connect_to_server(&self) -> Result<TCPServerTransport> {
        let mut transport = TCPServerTransport::new(self.server_addr);
        transport.connect()?;
        Ok(transport)
    }

    /// Open a new connection to the server, switched to this device.
    ///
    /// Prefers `transport_id` when set (unique even with duplicate serials), then falls back
    /// to `identifier` (serial), and finally to `transport-any` when neither is configured.
    pub(crate) fn connect_to_device(&self) -> Result<TCPServerTransport> {
        let cmd = match (self.transport_id, &self.identifier) {
            (Some(id), _) => ADBHostCommand::TransportId(id),
            (None, Some(serial)) => ADBHostCommand::TransportSerial(serial.clone()),
            (None, None) => ADBHostCommand::TransportAny,
        };

        let transport = self.connect_to_server()?;
        transport.send_adb_request(&ADBCommand::Host(cmd))?;
        Ok(transport)
    }
}
