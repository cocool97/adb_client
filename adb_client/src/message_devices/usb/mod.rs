#![doc = include_str!("./README.md")]

/// Common USB constants for Android Debug Bridge
pub mod constants {
    /// Standard Android vendor ID
    pub const ANDROID_VENDOR_ID: u16 = 0x18d1;

    /// Common ADB product IDs
    pub mod product_ids {
        /// ADB interface
        pub const ADB: u16 = 0x4ee7;
        /// ADB + MTP
        pub const ADB_MTP: u16 = 0x4ee2;
        /// ADB + RNDIS
        pub const ADB_RNDIS: u16 = 0x4ee4;
        /// Fastboot interface
        pub const FASTBOOT: u16 = 0x4ee0;
    }

    /// USB class codes for ADB detection
    pub mod class_codes {
        /// ADB subclass code
        pub const ADB_SUBCLASS: u8 = 0x42;
        /// ADB protocol code
        pub const ADB_PROTOCOL: u8 = 0x1;
        /// Bulk transfer class
        pub const BULK_CLASS: u8 = 0xdc;
        /// Bulk ADB subclass
        pub const BULK_ADB_SUBCLASS: u8 = 2;
    }
}

// ###################################################
// rusb specific modules
#[cfg(feature = "rusb")]
mod adb_rusb_device;

// Device implementations
#[cfg(feature = "rusb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rusb")))]
pub use adb_rusb_device::ADBRusbDevice;

// Transport implementations
#[cfg(feature = "rusb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rusb")))]
pub use backends::rusb_transport::RusbTransport;
// ###################################################

// ###################################################
// webusb specific modules
#[cfg(feature = "webusb")]
mod adb_webusb_device;

// Device implementations
#[cfg(feature = "webusb")]
#[cfg_attr(docsrs, doc(cfg(feature = "webusb")))]
pub use adb_webusb_device::ADBWebUsbDevice;

// Transport implementations
#[cfg(feature = "webusb")]
#[cfg_attr(docsrs, doc(cfg(feature = "webusb")))]
pub use backends::webusb_transport::WebUsbTransport;
// ###################################################

mod backends;
mod utils;

#[cfg(any(feature = "rusb", feature = "webusb"))]
mod adb_rsa_key;

#[cfg(any(feature = "rusb", feature = "webusb"))]
mod adb_usb_device;

// Utility functions
#[cfg(any(feature = "rusb", feature = "webusb"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "rusb", feature = "webusb"))))]
pub use utils::read_adb_private_key;

#[cfg(feature = "rusb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rusb")))]
pub use utils::{is_adb_device, search_adb_devices};
