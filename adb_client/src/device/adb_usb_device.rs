use rusb::Device;
use rusb::DeviceDescriptor;
use rusb::UsbContext;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use super::adb_message_device::ADBMessageDevice;
use super::models::MessageCommand;
use super::{ADBRsaKey, ADBTransportMessage};
use crate::device::adb_transport_message::{AUTH_RSAPUBLICKEY, AUTH_SIGNATURE, AUTH_TOKEN};
use crate::ADBDeviceExt;
use crate::ADBMessageTransport;
use crate::ADBTransport;
use crate::{Result, RustADBError, USBTransport};

/// Represent a device reached and available over USB.
#[derive(Debug)]
pub struct ADBUSBDevice {
    private_key: ADBRsaKey,
    inner: ADBMessageDevice<USBTransport>,
}

pub fn read_adb_private_key<P: AsRef<Path>>(private_key_path: P) -> Result<Option<ADBRsaKey>> {
    read_to_string(private_key_path.as_ref())
        .map_err(RustADBError::from)
        .map(|pk| match ADBRsaKey::new_from_pkcs8(&pk) {
            Ok(pk) => Some(pk),
            Err(e) => {
                log::error!("Error while create RSA private key: {e}");
                None
            }
        })
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
    const ADB_CLASS: u8 = 0xff;

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
                    && ((class == ADB_CLASS && subcl == ADB_SUBCLASS)
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
        let private_key = match read_adb_private_key(private_key_path)? {
            Some(pk) => pk,
            None => ADBRsaKey::new_random()?,
        };

        let mut s = Self {
            private_key,
            inner: ADBMessageDevice::new(USBTransport::new(vendor_id, product_id)),
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
            format!("host::{}\0", env!("CARGO_PKG_NAME"))
                .as_bytes()
                .to_vec(),
        );

        self.get_transport_mut().write_message(message)?;

        let message = self.get_transport_mut().read_message()?;

        // At this point, we should have received either:
        // - an AUTH message with arg0 == 1
        // - a CNXN message
        let auth_message = match message.header().command() {
            MessageCommand::Auth if message.header().arg0() == AUTH_TOKEN => message,
            MessageCommand::Auth if message.header().arg0() != AUTH_TOKEN => {
                return Err(RustADBError::ADBRequestFailed(
                    "Received AUTH message with type != 1".into(),
                ))
            }
            c => {
                return Err(RustADBError::ADBRequestFailed(format!(
                    "Wrong command received {}",
                    c
                )))
            }
        };

        let sign = self.private_key.sign(auth_message.into_payload())?;

        let message = ADBTransportMessage::new(MessageCommand::Auth, AUTH_SIGNATURE, 0, sign);

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

        let message = ADBTransportMessage::new(MessageCommand::Auth, AUTH_RSAPUBLICKEY, 0, pubkey);

        self.get_transport_mut().write_message(message)?;

        let response = self
            .get_transport_mut()
            .read_message_with_timeout(Duration::from_secs(10))?;

        match response.header().command() {
            MessageCommand::Cnxn => log::info!(
                "Authentication OK, device info {}",
                String::from_utf8(response.into_payload())?
            ),
            _ => {
                return Err(RustADBError::ADBRequestFailed(format!(
                    "wrong response {}",
                    response.header().command()
                )))
            }
        }

        Ok(())
    }

    fn get_transport_mut(&mut self) -> &mut USBTransport {
        self.inner.get_transport_mut()
    }
}

impl ADBDeviceExt for ADBUSBDevice {
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

impl Drop for ADBUSBDevice {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.get_transport_mut().disconnect();
    }
}
