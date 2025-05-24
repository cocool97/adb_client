#![crate_type = "lib"]
#![forbid(unsafe_code)]
#![forbid(missing_debug_implementations)]
#![forbid(missing_docs)]
#![doc = include_str!("../README.md")]

mod adb_device_ext;
mod constants;
mod device;
mod error;
mod models;
mod transports;

#[cfg(not(target_arch = "wasm32"))]
mod emulator_device;
#[cfg(not(target_arch = "wasm32"))]
mod mdns;

#[cfg(not(target_arch = "wasm32"))]
mod server;
#[cfg(not(target_arch = "wasm32"))]
mod server_device;
#[cfg(not(target_arch = "wasm32"))]
mod utils;

pub use adb_device_ext::ADBDeviceExt;
pub use device::ADBUSBDevice;
pub use error::{Result, RustADBError};
pub use models::{AdbStatResponse, RebootType};
pub use transports::{ADBMessageTransport, ADBTransport};

#[cfg(not(target_arch = "wasm32"))]
pub use device::ADBTcpDevice;
#[cfg(not(target_arch = "wasm32"))]
pub use emulator_device::ADBEmulatorDevice;
#[cfg(not(target_arch = "wasm32"))]
pub use mdns::*;
#[cfg(not(target_arch = "wasm32"))]
pub use server::*;
#[cfg(not(target_arch = "wasm32"))]
pub use server_device::ADBServerDevice;
#[cfg(not(target_arch = "wasm32"))]
pub use transports::*;
