#![crate_type = "lib"]
#![forbid(unsafe_code)]
#![forbid(missing_debug_implementations)]
#![forbid(missing_docs)]
#![doc = include_str!("../README.md")]

mod adb_device_ext;
mod constants;
mod emulator;
mod error;
mod models;
mod server;
mod transports;
mod usb;
mod utils;

pub use adb_device_ext::ADBDeviceExt;
pub use error::{Result, RustADBError};
pub use models::{AdbVersion, DeviceLong, DeviceShort, DeviceState, RebootType};
pub use server::*;
pub use transports::*;
pub use usb::ADBUSBDevice;
