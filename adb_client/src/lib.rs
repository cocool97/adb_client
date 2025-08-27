#![crate_type = "lib"]
#![forbid(unsafe_code)]
#![forbid(missing_debug_implementations)]
#![forbid(missing_docs)]
#![doc = include_str!("../README.md")]
// Feature `doc_cfg` is currently only available on nightly.
// It is activated when cfg `docsrs` is enabled.
// Documentation can be build locally using:
// `RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --all-features`
#![cfg_attr(docsrs, feature(doc_cfg))]

mod adb_device_ext;
mod constants;
mod device;
mod emulator_device;
mod error;

#[cfg(feature = "mdns")]
#[cfg_attr(docsrs, doc(cfg(feature = "mdns")))]
mod mdns;

mod models;
mod server;
mod server_device;
mod transports;
mod utils;

pub use adb_device_ext::ADBDeviceExt;
pub use device::ADBTcpDevice;

#[cfg(feature = "usb")]
#[cfg_attr(docsrs, doc(cfg(feature = "usb")))]
pub use device::{ADBUSBDevice, is_adb_device, search_adb_devices};

pub use emulator_device::ADBEmulatorDevice;
pub use error::{Result, RustADBError};

#[cfg(feature = "mdns")]
#[cfg_attr(docsrs, doc(cfg(feature = "mdns")))]
pub use mdns::*;

pub use models::{AdbStatResponse, RebootType};
pub use server::*;
pub use server_device::ADBServerDevice;
pub use transports::*;
