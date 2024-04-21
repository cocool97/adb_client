#![crate_type = "lib"]
#![forbid(unsafe_code)]
#![forbid(missing_debug_implementations)]
#![forbid(missing_docs)]
#![doc = include_str!("../README.md")]

mod adb_tcp_connection;
mod adb_termios;
mod commands;
mod error;
mod models;
pub use adb_tcp_connection::AdbTcpConnection;
pub use error::{Result, RustADBError};
pub use models::{AdbVersion, Device, DeviceLong, DeviceState, RebootType};
