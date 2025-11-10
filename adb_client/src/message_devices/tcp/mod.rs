#![doc = include_str!("./README.md")]

mod adb_tcp_device;
mod tcp_transport;

pub use adb_tcp_device::ADBTcpDevice;
