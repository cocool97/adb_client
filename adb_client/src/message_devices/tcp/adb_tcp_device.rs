use std::net::SocketAddr;
use std::path::Path;

use crate::Result;
use crate::message_devices::adb_message_device::ADBMessageDevice;
use crate::message_devices::adb_message_device_commands::impl_message_device_commands;
use crate::tcp::tcp_transport::TcpTransport;
use crate::utils::get_default_adb_key_path;

/// Represent a device reached and available over TCP.
#[derive(Debug)]
pub struct ADBTcpDevice {
    inner: ADBMessageDevice<TcpTransport>,
}

impl ADBTcpDevice {
    /// Instantiate a new [`ADBTcpDevice`]
    pub fn new<A: Into<SocketAddr>>(address: A) -> Result<Self> {
        Self::new_with_custom_private_key(address, get_default_adb_key_path()?)
    }

    /// Instantiate a new [`ADBTcpDevice`] using a custom private key path
    pub fn new_with_custom_private_key<P: AsRef<Path>, A: Into<SocketAddr>>(
        address: A,
        private_key_path: P,
    ) -> Result<Self> {
        Ok(Self {
            inner: ADBMessageDevice::new(
                TcpTransport::new(address, &private_key_path),
                private_key_path,
            )?,
        })
    }
}

impl_message_device_commands!(ADBTcpDevice);
