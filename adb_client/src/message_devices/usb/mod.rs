#![doc = include_str!("./README.md")]

mod adb_device_info;
mod adb_usb_device;
mod usb_transport;

#[cfg(feature = "webusb")]
#[cfg_attr(docsrs, doc(cfg(feature = "webusb")))]
mod webusb;

#[cfg(feature = "usb")]
#[cfg_attr(docsrs, doc(cfg(feature = "usb")))]
mod wired;

pub use adb_device_info::ADBDeviceInfo;
pub use adb_usb_device::ADBUSBDevice;
pub use usb_transport::USBTransport;

#[cfg(feature = "webusb")]
#[cfg_attr(docsrs, doc(cfg(feature = "webusb")))]
pub use webusb::WebUSBTransport;

#[cfg(feature = "usb")]
#[cfg_attr(docsrs, doc(cfg(feature = "usb")))]
pub use wired::WiredUSBTransport;
