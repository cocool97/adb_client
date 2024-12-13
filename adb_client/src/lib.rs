#![crate_type = "lib"]
#![forbid(unsafe_code)]
#![forbid(missing_debug_implementations)]
#![forbid(missing_docs)]
#![doc = include_str!("../README.md")]

mod adb_device_ext;
mod constants;
mod device;
mod emulator_device;
mod error;
mod mdns;
mod models;
mod server;
mod server_device;
mod transports;
mod utils;

pub use adb_device_ext::ADBDeviceExt;
pub use device::{ADBTcpDevice, ADBUSBDevice};
pub use emulator_device::ADBEmulatorDevice;
pub use error::{Result, RustADBError};
pub use mdns::*;
pub use models::{AdbStatResponse, RebootType};
pub use server::*;
pub use server_device::ADBServerDevice;
pub use transports::*;
