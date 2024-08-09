#![crate_type = "lib"]
#![forbid(unsafe_code)]
#![forbid(missing_debug_implementations)]
#![forbid(missing_docs)]
#![doc = include_str!("../../README.md")]

mod adb_termios;
mod emulator;
mod error;
mod models;
mod server;
mod transports;
mod utils;

pub use error::{Result, RustADBError};
pub use models::{AdbVersion, DeviceLong, DeviceShort, DeviceState, RebootType};
pub use server::*;
pub use transports::*;
