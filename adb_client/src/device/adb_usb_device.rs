use rusb::Device;
use rusb::DeviceDescriptor;
use rusb::UsbContext;
use rusb::constants::LIBUSB_CLASS_VENDOR_SPEC;
use std::fs::read_to_string;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use super::adb_message_device::ADBMessageDevice;
use super::models::MessageCommand;
use super::{ADBRsaKey, ADBTransportMessage};
use crate::ADBDeviceExt;
use crate::ADBMessageTransport;
use crate::ADBTransport;
use crate::device::adb_transport_message::{AUTH_RSAPUBLICKEY, AUTH_SIGNATURE, AUTH_TOKEN};
use crate::{Result, RustADBError, USBTransport};

pub fn read_adb_private_key<P: AsRef<Path>>(private_key_path: P) -> Result<Option<ADBRsaKey>> {
    Ok(read_to_string(private_key_path.as_ref()).map(|pk| {
        match ADBRsaKey::new_from_pkcs8(&pk) {
            Ok(pk) => Some(pk),
            Err(e) => {
                log::error!("Error while create RSA private key: {e}");
                None
            }
        }
    })?)
}

/// Search for adb devices with known interface class and subclass values
fn search_adb_devices() -> Result<Option<(u16, u16)>> {
    let mut found_devices = vec![];
    for device in rusb::devices()?.iter() {
        let Ok(des) = device.device_descriptor() else {
            continue;
        };
        if is_adb_device(&device, &des) {
            log::debug!(
                "Autodetect device {:04x}:{:04x}",
                des.vendor_id(),
                des.product_id()
            );
            found_devices.push((des.vendor_id(), des.product_id()));
        }
    }

    match (found_devices.first(), found_devices.get(1)) {
        (None, _) => Ok(None),
        (Some(identifiers), None) => Ok(Some(*identifiers)),
        (Some((vid1, pid1)), Some((vid2, pid2))) => Err(RustADBError::DeviceNotFound(format!(
            "Found two Android devices {:04x}:{:04x} and {:04x}:{:04x}",
            vid1, pid1, vid2, pid2
        ))),
    }
}

fn is_adb_device<T: UsbContext>(device: &Device<T>, des: &DeviceDescriptor) -> bool {
    const ADB_SUBCLASS: u8 = 0x42;
    const ADB_PROTOCOL: u8 = 0x1;

    // Some devices require choosing the file transfer mode
    // for usb debugging to take effect.
    const BULK_CLASS: u8 = 0xdc;
    const BULK_ADB_SUBCLASS: u8 = 2;

    for n in 0..des.num_configurations() {
        let Ok(config_des) = device.config_descriptor(n) else {
            continue;
        };
        for interface in config_des.interfaces() {
            for interface_des in interface.descriptors() {
                let proto = interface_des.protocol_code();
                let class = interface_des.class_code();
                let subcl = interface_des.sub_class_code();
                if proto == ADB_PROTOCOL
                    && ((class == LIBUSB_CLASS_VENDOR_SPEC && subcl == ADB_SUBCLASS)
                        || (class == BULK_CLASS && subcl == BULK_ADB_SUBCLASS))
                {
                    return true;
                }
            }
        }
    }
    false
}

pub fn get_default_adb_key_path() -> Result<PathBuf> {
    homedir::my_home()
        .ok()
        .flatten()
        .map(|home| home.join(".android").join("adbkey"))
        .ok_or(RustADBError::NoHomeDirectory)
}

/// Represent a device reached and available over USB.
#[derive(Debug)]
pub struct ADBUSBDevice {
    private_key: ADBRsaKey,
    inner: ADBMessageDevice<USBTransport>,
}

impl ADBUSBDevice {
    /// Instantiate a new [`ADBUSBDevice`]
    pub fn new(vendor_id: u16, product_id: u16) -> Result<Self> {
        Self::new_with_custom_private_key(vendor_id, product_id, get_default_adb_key_path()?)
    }

    /// Instantiate a new [`ADBUSBDevice`] using a custom private key path
    pub fn new_with_custom_private_key(
        vendor_id: u16,
        product_id: u16,
        private_key_path: PathBuf,
    ) -> Result<Self> {
        Self::new_from_transport_inner(USBTransport::new(vendor_id, product_id)?, private_key_path)
    }

    /// Instantiate a new [`ADBUSBDevice`] from a [`USBTransport`] and an optional private key path.
    pub fn new_from_transport(
        transport: USBTransport,
        private_key_path: Option<PathBuf>,
    ) -> Result<Self> {
        let private_key_path = match private_key_path {
            Some(private_key_path) => private_key_path,
            None => get_default_adb_key_path()?,
        };

        Self::new_from_transport_inner(transport, private_key_path)
    }

    fn new_from_transport_inner(
        transport: USBTransport,
        private_key_path: PathBuf,
    ) -> Result<Self> {
        let private_key = match read_adb_private_key(private_key_path)? {
            Some(pk) => pk,
            None => ADBRsaKey::new_random()?,
        };

        let mut s = Self {
            private_key,
            inner: ADBMessageDevice::new(transport),
        };

        s.connect()?;

        Ok(s)
    }

    /// autodetect connected ADB devices and establish a connection with the first device found
    pub fn autodetect() -> Result<Self> {
        Self::autodetect_with_custom_private_key(get_default_adb_key_path()?)
    }

    /// autodetect connected ADB devices and establish a connection with the first device found using a custom private key path
    pub fn autodetect_with_custom_private_key(private_key_path: PathBuf) -> Result<Self> {
        match search_adb_devices()? {
            Some((vendor_id, product_id)) => {
                ADBUSBDevice::new_with_custom_private_key(vendor_id, product_id, private_key_path)
            }
            _ => Err(RustADBError::DeviceNotFound(
                "cannot find USB devices matching the signature of an ADB device".into(),
            )),
        }
    }

    /// Send initial connect
    pub fn connect(&mut self) -> Result<()> {
        self.get_transport_mut().connect()?;

        let message = ADBTransportMessage::new(
            MessageCommand::Cnxn,
            0x01000000,
            1048576,
            format!("host::{}\0", env!("CARGO_PKG_NAME")).as_bytes(),
        );

        self.get_transport_mut().write_message(message)?;

        let message = self.get_transport_mut().read_message()?;
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
    fn get_transport_mut(&mut self) -> &mut USBTransport {
        self.inner.get_transport_mut()
    }
}

impl ADBDeviceExt for ADBUSBDevice {
    #[inline]
    fn shell_command(&mut self, command: &[&str], output: &mut dyn Write) -> Result<()> {
        self.inner.shell_command(command, output)
    }

    #[inline]
    fn shell<'a>(&mut self, reader: &mut dyn Read, writer: Box<(dyn Write + Send)>) -> Result<()> {
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

impl Drop for ADBUSBDevice {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.get_transport_mut().disconnect();
    }
}
