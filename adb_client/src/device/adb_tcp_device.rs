use std::io::Write;
use std::path::{Path, PathBuf};
use std::{io::Read, net::SocketAddr};

use super::adb_message_device::ADBMessageDevice;
use super::models::MessageCommand;
use super::{ADBRsaKey, ADBTransportMessage, get_default_adb_key_path};
use crate::device::adb_usb_device::read_adb_private_key;
use crate::{ADBDeviceExt, ADBMessageTransport, ADBTransport, Result, TcpTransport};

/// Represent a device reached and available over USB.
#[derive(Debug)]
pub struct ADBTcpDevice {
    private_key: ADBRsaKey,
    inner: ADBMessageDevice<TcpTransport>,
}

impl ADBTcpDevice {
    /// Instantiate a new [`ADBTcpDevice`]
    pub fn new(address: SocketAddr) -> Result<Self> {
        Self::new_with_custom_private_key(address, get_default_adb_key_path()?)
    }

    /// Instantiate a new [`ADBTcpDevice`] using a custom private key path
    pub fn new_with_custom_private_key(
        address: SocketAddr,
        private_key_path: PathBuf,
    ) -> Result<Self> {
        let private_key = if let Some(private_key) = read_adb_private_key(&private_key_path)? {
            private_key
        } else {
            log::warn!(
                "No private key found at path {}. Using a temporary random one.",
                private_key_path.display()
            );
            ADBRsaKey::new_random()?
        };

        let mut device = Self {
            private_key,
            inner: ADBMessageDevice::new(TcpTransport::new(address)?),
        };

        device.connect()?;

        Ok(device)
    }

    /// Send initial connect
    pub fn connect(&mut self) -> Result<()> {
        self.get_transport_mut().connect()?;

        let message = ADBTransportMessage::new(
            MessageCommand::Cnxn,
            0x0100_0000,
            1_048_576,
            format!("host::{}\0", env!("CARGO_PKG_NAME")).as_bytes(),
        );

        self.get_transport_mut().write_message(message)?;

        let message = self.get_transport_mut().read_message()?;

        // Check if a client is requesting a secure connection and upgrade it if necessary
        match message.header().command() {
            MessageCommand::Stls => {
                self.get_transport_mut()
                    .write_message(ADBTransportMessage::new(MessageCommand::Stls, 1, 0, &[]))?;
                self.get_transport_mut().upgrade_connection()?;
                log::debug!("Connection successfully upgraded from TCP to TLS");
                Ok(())
            }
            MessageCommand::Cnxn => {
                log::debug!("Unencrypted connection established");
                Ok(())
            }
            MessageCommand::Auth => {
                log::debug!("Authentication required");
                self.inner.auth_handshake(message, &self.private_key)
            }
            _ => Err(crate::RustADBError::WrongResponseReceived(
                "Expected CNXN, STLS or AUTH command".to_string(),
                message.header().command().to_string(),
            )),
        }
    }

    #[inline]
    fn get_transport_mut(&mut self) -> &mut TcpTransport {
        self.inner.get_transport_mut()
    }
}

impl ADBDeviceExt for ADBTcpDevice {
    #[inline]
    fn shell_command(&mut self, command: &[&str], output: &mut dyn Write) -> Result<()> {
        self.inner.shell_command(command, output)
    }

    #[inline]
    fn shell(&mut self, reader: &mut dyn Read, writer: Box<dyn Write + Send>) -> Result<()> {
        self.inner.shell(reader, writer)
    }

    #[inline]
    fn stat(&mut self, remote_path: &str) -> Result<crate::AdbStatResponse> {
        self.inner.stat(remote_path)
    }

    #[inline]
    fn pull(&mut self, source: &dyn AsRef<str>, output: &mut dyn Write) -> Result<()> {
        self.inner.pull(source, output)
    }

    #[inline]
    fn push(&mut self, stream: &mut dyn Read, path: &dyn AsRef<str>) -> Result<()> {
        self.inner.push(stream, path)
    }

    #[inline]
    fn reboot(&mut self, reboot_type: crate::RebootType) -> Result<()> {
        self.inner.reboot(reboot_type)
    }

    #[inline]
    fn install(&mut self, apk_path: &dyn AsRef<Path>) -> Result<()> {
        self.inner.install(apk_path)
    }

    #[inline]
    fn uninstall(&mut self, package: &str) -> Result<()> {
        self.inner.uninstall(package)
    }

    #[inline]
    fn framebuffer_inner(&mut self) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
        self.inner.framebuffer_inner()
    }
}

impl Drop for ADBTcpDevice {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.get_transport_mut().disconnect();
    }
}
