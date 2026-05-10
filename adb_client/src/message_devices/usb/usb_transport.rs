use crate::{
    Result, adb_transport::ADBTransport,
    message_devices::adb_message_transport::ADBMessageTransport, usb::ADBDeviceInfo,
};

/// Trait representing a USB transport layer for ADB devices.
pub trait USBTransport: ADBTransport + ADBMessageTransport {
    /// Find and return a list of all connected Android devices with known interface class and subclass values
    fn find_all_connected_adb_devices() -> Result<Vec<ADBDeviceInfo>>;

    /// Return the vendor ID of the underlying USB device
    fn vendor_id(&self) -> u16;

    /// Return the product ID of the underlying USB device
    fn product_id(&self) -> u16;
}
