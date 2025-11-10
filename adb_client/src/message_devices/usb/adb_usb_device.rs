use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::ADBDeviceExt;
use crate::AUTH_TOKEN;
use crate::Result;
use crate::RustADBError;
use crate::message_devices::AUTH_RSAPUBLICKEY;
use crate::message_devices::AUTH_SIGNATURE;
use crate::message_devices::adb_message_device::ADBMessageDevice;
use crate::message_devices::adb_message_transport::ADBMessageTransport;
use crate::message_devices::adb_rsa_key::ADBRsaKey;
use crate::message_devices::adb_transport_message::ADBTransportMessage;
use crate::message_devices::message_commands::MessageCommand;
use crate::usb::read_adb_private_key;
use crate::utils::get_default_adb_key_path;

/// Private struct implementing Android USB device logic, depending on a `ADBMessageTransport`.
#[derive(Debug)]
pub(crate) struct ADBUSBDevice<T: ADBMessageTransport> {
    private_key: ADBRsaKey,
    inner: ADBMessageDevice<T>,
}

impl<T: ADBMessageTransport> ADBUSBDevice<T> {
    /// Instantiate a new [`ADBUSBDevice`] from a [`RusbTransport`] and an optional private key path.
    pub fn new_from_transport(transport: T, private_key_path: Option<PathBuf>) -> Result<Self> {
        let private_key_path = match private_key_path {
            Some(private_key_path) => private_key_path,
            None => get_default_adb_key_path()?,
        };

        Self::new_from_transport_inner(transport, &private_key_path)
    }

    fn new_from_transport_inner(transport: T, private_key_path: &PathBuf) -> Result<Self> {
        let private_key = if let Some(private_key) = read_adb_private_key(private_key_path)? {
            private_key
        } else {
            log::warn!(
                "No private key found at path {}. Using a temporary random one.",
                private_key_path.display()
            );
            ADBRsaKey::new_random()?
        };

        let mut s = Self {
            private_key,
            inner: ADBMessageDevice::new(transport),
        };

        s.connect()?;

        Ok(s)
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
        // If the device returned CNXN instead of AUTH it does not require authentication,
        // so we can skip the auth steps.
        if message.header().command() == MessageCommand::Cnxn {
            return Ok(());
        }
        message.assert_command(MessageCommand::Auth)?;

        // At this point, we should have receive an AUTH message with arg0 == 1
        let auth_message = match message.header().arg0() {
            AUTH_TOKEN => message,
            v => {
                return Err(RustADBError::ADBRequestFailed(format!(
                    "Received AUTH message with type != 1 ({v})"
                )));
            }
        };

        let sign = self.private_key.sign(auth_message.into_payload())?;

        let message = ADBTransportMessage::new(MessageCommand::Auth, AUTH_SIGNATURE, 0, &sign);

        self.get_transport_mut().write_message(message)?;

        let received_response = self.get_transport_mut().read_message()?;

        if received_response.header().command() == MessageCommand::Cnxn {
            log::info!(
                "Authentication OK, device info {}",
                String::from_utf8(received_response.into_payload())?
            );
            return Ok(());
        }

        let mut pubkey = self.private_key.android_pubkey_encode()?.into_bytes();
        pubkey.push(b'\0');

        let message = ADBTransportMessage::new(MessageCommand::Auth, AUTH_RSAPUBLICKEY, 0, &pubkey);

        self.get_transport_mut().write_message(message)?;

        let response = self
            .get_transport_mut()
            .read_message_with_timeout(Duration::from_secs(10))
            .and_then(|message| {
                message.assert_command(MessageCommand::Cnxn)?;
                Ok(message)
            })?;

        log::info!(
            "Authentication OK, device info {}",
            String::from_utf8(response.into_payload())?
        );

        Ok(())
    }

    #[inline]
    pub(crate) fn get_transport_mut(&mut self) -> &mut T {
        self.inner.get_transport_mut()
    }
}

impl<T: ADBMessageTransport> ADBDeviceExt for ADBUSBDevice<T> {
    #[inline]
    fn shell_command(&mut self, command: &[&str], output: &mut dyn Write) -> Result<()> {
        self.inner.shell_command(command, output)
    }

    #[inline]
    fn shell<'a>(&mut self, reader: &mut dyn Read, writer: Box<dyn Write + Send>) -> Result<()> {
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

impl<T: ADBMessageTransport> Drop for ADBUSBDevice<T> {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.get_transport_mut().disconnect();
    }
}
