use crate::Result;

/// Trait representing a transport usable by ADB protocol. It must both implement [`ADBConnectableTransport`] and [`ADBDisonnectableTransport`].
pub trait ADBTransport: ADBConnectableTransport + ADBDisconnectableTransport {}

/// Trait representing a transport that can be connected.
pub trait ADBConnectableTransport {
    /// Initializes the connection to this transport.
    fn connect(&mut self) -> Result<()>;
}

/// Trait representing a transport that can be disconnected.
pub trait ADBDisconnectableTransport {
    /// Shuts down the connection to this transport.
    fn disconnect(&mut self) -> Result<()>;
}

/// Internal `TypeState` representing a connected transport.
#[derive(Debug)]
pub struct Connected;

/// Internal `TypeState` representing a disconnected transport.
#[derive(Debug)]
pub struct NotConnected;
