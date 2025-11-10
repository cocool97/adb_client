//! USB utilities that are independent of specific transport implementations

use crate::message_devices::adb_rsa_key::ADBRsaKey;
use crate::{Result, RustADBError};
use std::fs::read_to_string;
use std::path::Path;

#[cfg(feature = "rusb")]
use rusb::{Device, DeviceDescriptor, UsbContext};

#[cfg(feature = "rusb")]
use rusb::constants::LIBUSB_CLASS_VENDOR_SPEC;

use crate::usb::constants::class_codes::{
    ADB_PROTOCOL, ADB_SUBCLASS, BULK_ADB_SUBCLASS, BULK_CLASS,
};

/// Read an ADB private key from a file path
///
/// Returns `Ok(None)` if the file doesn't exist, `Ok(Some(key))` if the key was successfully loaded,
/// or an error if there was a problem reading the file.
pub fn read_adb_private_key<P: AsRef<Path>>(private_key_path: P) -> Result<Option<ADBRsaKey>> {
    // Try to read the private key file from given path
    // If the file is not found, return None
    // If there is another error while reading the file, return this error
    // Else, return the private key content
    let pk = match read_to_string(private_key_path.as_ref()) {
        Ok(pk) => pk,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    match ADBRsaKey::new_from_pkcs8(&pk) {
        Ok(pk) => Ok(Some(pk)),
        Err(e) => Err(e),
    }
}

/// Search for ADB devices connected via USB
///
/// Returns the vendor_id and product_id of the first ADB device found,
/// or `None` if no devices are found.
#[cfg(feature = "rusb")]
pub fn search_adb_devices() -> Result<Option<(u16, u16)>> {
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
            "Found two Android devices {vid1:04x}:{pid1:04x} and {vid2:04x}:{pid2:04x}",
        ))),
    }
}

/// Check if a USB device is an ADB device
///
/// This function inspects the device descriptor and configuration to determine
/// if it's an Android Debug Bridge device.
#[cfg(feature = "rusb")]
pub fn is_adb_device<T: UsbContext>(device: &Device<T>, des: &DeviceDescriptor) -> bool {
    // Some devices require choosing the file transfer mode
    // for usb debugging to take effect.
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
