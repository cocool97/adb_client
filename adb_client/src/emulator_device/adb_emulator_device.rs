use std::net::{Ipv4Addr, SocketAddrV4};

use crate::{ADBServerDevice, ADBTransport, Result, RustADBError, TCPEmulatorTransport};

#[must_use]
fn get_emulator_port(id: &str) -> Option<&str> {
    const PREFIX: &str = "emulator-";
    if id.len() <= PREFIX.len() || !id.starts_with(PREFIX) {
        return None;
    }
    // bounds-check already done above
    let suffix = &id[PREFIX.len()..];
    if suffix.bytes().all(|c| c.is_ascii_digit()) {
        return Some(suffix);
    }
    None
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
            None => Ipv4Addr::LOCALHOST,
        };

        let port = identifier
            .lines()
            .find_map(|id| {
                const PREFIX: &str = "emulator-";
                if id.len() <= PREFIX.len() || !id.starts_with(PREFIX) {
                    return None;
                }
                // bounds-check already done above
                Some(&id[PREFIX.len()..])
            })
            .ok_or(RustADBError::DeviceNotFound(format!(
                "Device {} is likely not an emulator",
                identifier
            )))?
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
        match &value.identifier {
            Some(device_identifier) => ADBEmulatorDevice::new(
                device_identifier.clone(),
                Some(*value.transport.get_socketaddr().ip()),
            ),
            None => Err(RustADBError::DeviceNotFound(
                "cannot connect to an emulator device without knowing its identifier".to_string(),
            )),
        }
    }
}

impl Drop for ADBEmulatorDevice {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.transport.disconnect();
    }
}
