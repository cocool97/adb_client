use std::net::SocketAddr;
use std::path::Path;

use super::adb_message_device::ADBMessageDevice;
use super::models::MessageCommand;
use super::ADBTransportMessage;
use crate::{ADBDeviceExt, ADBMessageTransport, ADBTransport, Result, RustADBError, TcpTransport};

/// Represent a device reached and available over USB.
#[derive(Debug)]
pub struct ADBTcpDevice {
    inner: ADBMessageDevice<TcpTransport>,
}

impl ADBTcpDevice {
    /// Instantiate a new [`ADBTcpDevice`]
    pub fn new(address: SocketAddr) -> Result<Self> {
        let mut device = Self {
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
            0x01000000,
            1048576,
            format!("host::{}\0", env!("CARGO_PKG_NAME"))
                .as_bytes()
                .to_vec(),
        );

        self.get_transport_mut().write_message(message)?;

        let message = self.get_transport_mut().read_message()?;

        // At this point, we should have received a STLS message
        if message.header().command() != MessageCommand::Stls {
            return Err(RustADBError::ADBRequestFailed(format!(
                "Wrong command received {}",
                message.header().command()
            )));
        };

        let message = ADBTransportMessage::new(MessageCommand::Stls, 1, 0, vec![]);

        self.get_transport_mut().write_message(message)?;

        // Upgrade TCP connection to TLS
        self.get_transport_mut().upgrade_connection()?;

        log::debug!("Connection successfully upgraded from TCP to TLS");

        Ok(())
    }

    fn get_transport_mut(&mut self) -> &mut TcpTransport {
        self.inner.get_transport_mut()
    }
}

impl ADBDeviceExt for ADBTcpDevice {
    fn shell_command<S: ToString, W: std::io::Write>(
        &mut self,
        command: impl IntoIterator<Item = S>,
        output: W,
    ) -> Result<()> {
        self.inner.shell_command(command, output)
    }

    fn shell<R: std::io::Read, W: std::io::Write + Send + 'static>(
        &mut self,
        reader: R,
        writer: W,
    ) -> Result<()> {
        self.inner.shell(reader, writer)
    }

    fn stat(&mut self, remote_path: &str) -> Result<crate::AdbStatResponse> {
        self.inner.stat(remote_path)
    }

    fn pull<A: AsRef<str>, W: std::io::Write>(&mut self, source: A, output: W) -> Result<()> {
        self.inner.pull(source, output)
    }

    fn push<R: std::io::Read, A: AsRef<str>>(&mut self, stream: R, path: A) -> Result<()> {
        self.inner.push(stream, path)
    }

    fn reboot(&mut self, reboot_type: crate::RebootType) -> Result<()> {
        self.inner.reboot(reboot_type)
    }

    fn install<P: AsRef<Path>>(&mut self, apk_path: P) -> Result<()> {
        self.inner.install(apk_path)
    }
}

impl Drop for ADBTcpDevice {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.get_transport_mut().disconnect();
    }
}
