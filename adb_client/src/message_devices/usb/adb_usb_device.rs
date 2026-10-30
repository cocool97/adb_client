use std::path::Path;
use std::path::PathBuf;

use crate::Result;
use crate::RustADBError;
use crate::message_devices::adb_message_device::ADBMessageDevice;
use crate::message_devices::adb_message_device_commands::impl_message_device_commands;
use crate::usb::usb_transport::USBTransport;
use crate::usb::utils;
use crate::utils::get_default_adb_key_path;

/// Represent a device reached and available over USB.
#[derive(Debug)]
pub struct ADBUSBDevice {
    inner: ADBMessageDevice<USBTransport>,
    vendor_id: u16,
    product_id: u16,
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
        let vendor_id = transport.vendor_id()?;
        let product_id = transport.product_id()?;

        Ok(Self {
            inner: ADBMessageDevice::new(transport, private_key_path)?,
            vendor_id,
            product_id,
        })
    }

    /// Returns the vendor ID of the device
    #[must_use]
    pub const fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    /// Returns the product ID of the device
    #[must_use]
    pub const fn product_id(&self) -> u16 {
        self.product_id
    }

    /// Autodetect connected ADB devices and establish a connection with the first device found
    ///
    /// # Errors
    ///
    /// Returns an error if multiple devices or none are connected.
    pub fn autodetect() -> Result<Self> {
        Self::autodetect_with_custom_private_key(get_default_adb_key_path()?)
    }

    /// Autodetect connected ADB devices and establish a connection with the first device found using a custom private key path
    ///
    /// # Errors
    ///
    /// Returns an error if multiple devices are connected or if none can be detected.
    pub fn autodetect_with_custom_private_key(private_key_path: PathBuf) -> Result<Self> {
        match utils::get_single_connected_adb_device()? {
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
}

impl_message_device_commands!(ADBUSBDevice);
