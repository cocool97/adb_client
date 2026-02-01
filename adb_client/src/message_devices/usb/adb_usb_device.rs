use rusb::Device;
use rusb::DeviceDescriptor;
use rusb::UsbContext;
use rusb::constants::LIBUSB_CLASS_VENDOR_SPEC;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::ADBDeviceExt;
use crate::ADBListItemType;
use crate::Result;
use crate::RustADBError;
use crate::adb_transport::ADBTransport;
use crate::message_devices::adb_message_device::ADBMessageDevice;
use crate::message_devices::adb_message_transport::ADBMessageTransport;
use crate::message_devices::adb_transport_message::ADBTransportMessage;
use crate::message_devices::message_commands::MessageCommand;
use crate::message_devices::models::ADBRsaKey;
use crate::message_devices::models::read_adb_private_key;
use crate::models::RemountInfo;
use crate::usb::usb_transport::USBTransport;
use crate::utils::get_default_adb_key_path;

/// Represents an Android device connected via USB
#[derive(Clone, Debug)]
pub struct ADBDeviceInfo {
    /// Vendor ID of the device
    pub vendor_id: u16,
    /// Product ID of the device
    pub product_id: u16,
    /// Textual description of the device
    pub device_description: String,
}

/// Find and return a list of all connected Android devices with known interface class and subclass values
pub fn find_all_connected_adb_devices() -> Result<Vec<ADBDeviceInfo>> {
    let mut found_devices = vec![];

    for device in rusb::devices()?.iter() {
        let Ok(des) = device.device_descriptor() else {
            continue;
        };

        if is_adb_device(&device, &des) {
            let Ok(device_handle) = device.open() else {
                found_devices.push(ADBDeviceInfo {
                    vendor_id: des.vendor_id(),
                    product_id: des.product_id(),
                    device_description: "Unknown device".to_string(),
                });
                continue;
            };

            let manufacturer = device_handle
                .read_manufacturer_string_ascii(&des)
                .unwrap_or_else(|_| "Unknown".to_string());

            let product = device_handle
                .read_product_string_ascii(&des)
                .unwrap_or_else(|_| "Unknown".to_string());

            found_devices.push(ADBDeviceInfo {
                vendor_id: des.vendor_id(),
                product_id: des.product_id(),
                device_description: format!("{manufacturer} {product}"),
            });
        }
    }

    Ok(found_devices)
}

/// Find and return an USB-connected Android device with known interface class and subclass values.
///
/// Returns the first device found or None if no device is found.
/// If multiple devices are found, an error is returned.
pub fn get_single_connected_adb_device() -> Result<Option<ADBDeviceInfo>> {
    let found_devices = find_all_connected_adb_devices()?;
    match (found_devices.first(), found_devices.get(1)) {
        (None, _) => Ok(None),
        (Some(device_info), None) => {
            log::debug!(
                "Autodetect device {:04x}:{:04x} - {}",
                device_info.vendor_id,
                device_info.product_id,
                device_info.device_description
            );
            Ok(Some(device_info.clone()))
        }
        (Some(device_1), Some(device_2)) => Err(RustADBError::DeviceNotFound(format!(
            "Found two Android devices {:04x}:{:04x} and {:04x}:{:04x}",
            device_1.vendor_id, device_1.product_id, device_2.vendor_id, device_2.product_id
        ))),
    }
}

/// Check whether a device with given descriptor is an ADB device
pub fn is_adb_device<T: UsbContext>(device: &Device<T>, des: &DeviceDescriptor) -> bool {
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
    pub fn new_with_custom_private_key<P: AsRef<Path>>(
        vendor_id: u16,
        product_id: u16,
        private_key_path: P,
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

        Self::new_from_transport_inner(transport, &private_key_path)
    }

    fn new_from_transport_inner<P: AsRef<Path>>(
        transport: USBTransport,
        private_key_path: P,
    ) -> Result<Self> {
        let private_key = if let Some(private_key) = read_adb_private_key(&private_key_path)? {
            private_key
        } else {
            log::warn!(
                "No private key found at path {}. Using a temporary random one.",
                private_key_path.as_ref().display()
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

    /// autodetect connected ADB devices and establish a connection with the first device found
    pub fn autodetect() -> Result<Self> {
        Self::autodetect_with_custom_private_key(get_default_adb_key_path()?)
    }

    /// autodetect connected ADB devices and establish a connection with the first device found using a custom private key path
    pub fn autodetect_with_custom_private_key(private_key_path: PathBuf) -> Result<Self> {
        match get_single_connected_adb_device()? {
            Some(device_info) => Self::new_with_custom_private_key(
                device_info.vendor_id,
                device_info.product_id,
                private_key_path,
            ),
            _ => Err(RustADBError::DeviceNotFound(
                "cannot find USB devices matching the signature of an ADB device".into(),
            )),
        }
    }

    /// Send initial connect
    pub fn connect(&mut self) -> Result<()> {
        self.get_transport_mut().connect()?;

        let message = ADBTransportMessage::try_new(
            MessageCommand::Cnxn,
            0x0100_0000,
            1_048_576,
            format!("host::{}\0", env!("CARGO_PKG_NAME")).as_bytes(),
        )?;

        self.get_transport_mut().write_message(message)?;

        let message = self.get_transport_mut().read_message()?;
        // If the device returned CNXN instead of AUTH it does not require authentication,
        // so we can skip the auth steps.
        if message.header().command() == MessageCommand::Cnxn {
            return Ok(());
        }

        message.assert_command(MessageCommand::Auth)?;
        self.inner.auth_handshake(message, &self.private_key)
    }

    #[inline]
    fn get_transport_mut(&mut self) -> &mut USBTransport {
        self.inner.get_transport_mut()
    }
}

impl ADBDeviceExt for ADBUSBDevice {
    #[inline]
    fn shell_command(
        &mut self,
        command: &dyn AsRef<str>,
        stdout: Option<&mut dyn Write>,
        stderr: Option<&mut dyn Write>,
    ) -> Result<Option<u8>> {
        self.inner.shell_command(command, stdout, stderr)
    }

    #[inline]
    fn shell<'a>(&mut self, reader: &mut dyn Read, writer: Box<dyn Write + Send>) -> Result<()> {
        self.inner.shell(reader, writer)
    }

    #[inline]
    fn stat(&mut self, remote_path: &dyn AsRef<str>) -> Result<crate::AdbStatResponse> {
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
    fn remount(&mut self) -> Result<Vec<RemountInfo>> {
        self.inner.remount()
    }

    #[inline]
    fn root(&mut self) -> Result<()> {
        self.inner.root()
    }

    #[inline]
    fn install(&mut self, apk_path: &dyn AsRef<Path>, user: Option<&str>) -> Result<()> {
        self.inner.install(apk_path, user)
    }

    #[inline]
    fn uninstall(&mut self, package: &dyn AsRef<str>, user: Option<&str>) -> Result<()> {
        self.inner.uninstall(package, user)
    }

    #[inline]
    fn enable_verity(&mut self) -> Result<()> {
        self.inner.enable_verity()
    }

    #[inline]
    fn disable_verity(&mut self) -> Result<()> {
        self.inner.disable_verity()
    }

    #[inline]
    fn framebuffer_inner(&mut self) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
        self.inner.framebuffer_inner()
    }

    #[inline]
    fn list(&mut self, path: &dyn AsRef<str>) -> Result<Vec<ADBListItemType>> {
        self.inner.list(path)
    }

    #[inline]
    fn exec(
        &mut self,
        command: &str,
        reader: &mut dyn Read,
        writer: Box<dyn Write + Send>,
    ) -> Result<()> {
        self.inner.exec(command, reader, writer)
    }
}

impl Drop for ADBUSBDevice {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.get_transport_mut().disconnect();
    }
}
