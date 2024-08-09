use std::net::{Ipv4Addr, SocketAddrV4};

use crate::{ADBTransport, Result, RustADBError, TCPEmulatorTransport};
use lazy_static::lazy_static;
use regex::Regex;

use super::ADBServerDevice;

lazy_static! {
    pub static ref EMULATOR_REGEX: Regex =
        Regex::new("^emulator-(?P<port>\\d+)$").expect("wrong syntax for emulator regex");
}

/// Represents an emulator connected to the ADB server.
#[derive(Debug)]
pub struct ADBEmulatorDevice {
    /// Unique device identifier.
    pub identifier: String,
    /// Internal [TCPEmulatorTransport]
    transport: TCPEmulatorTransport,
}

impl ADBEmulatorDevice {
    /// Instantiates a new [ADBEmulatorDevice]
    pub fn new(identifier: String, ip_address: Option<Ipv4Addr>) -> Result<Self> {
        let ip_address = match ip_address {
            Some(ip_address) => ip_address,
            None => Ipv4Addr::new(127, 0, 0, 1),
        };

        let groups = EMULATOR_REGEX
            .captures(&identifier)
            .ok_or(RustADBError::DeviceNotFound(format!(
                "Device {} is likely not an emulator",
                identifier
            )))?;

        let port = groups
            .name("port")
            .ok_or(RustADBError::RegexParsingError)?
            .as_str()
            .parse::<u16>()?;

        let socket_addr = SocketAddrV4::new(ip_address, port);

        let transport = TCPEmulatorTransport::new(socket_addr);
        Ok(Self {
            identifier,
            transport,
        })
    }

    pub(crate) fn get_transport_mut(&mut self) -> &mut TCPEmulatorTransport {
        &mut self.transport
    }

    /// Connect to underlying transport
    pub(crate) fn connect(&mut self) -> Result<&mut TCPEmulatorTransport> {
        self.transport.connect()?;

        Ok(self.get_transport_mut())
    }
}

impl TryFrom<ADBServerDevice> for ADBEmulatorDevice {
    type Error = RustADBError;

    fn try_from(value: ADBServerDevice) -> std::result::Result<Self, Self::Error> {
        ADBEmulatorDevice::new(
            value.identifier.clone(),
            Some(*value.get_transport().get_socketaddr().ip()),
        )
    }
}

impl Drop for ADBEmulatorDevice {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.transport.disconnect();
    }
}
