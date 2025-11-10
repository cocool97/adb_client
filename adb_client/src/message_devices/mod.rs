/// USB-related definitions
pub mod usb;

/// Device reachable over TCP related definition
pub mod tcp;

pub(crate) const AUTH_TOKEN: u32 = 1;
pub(crate) const AUTH_SIGNATURE: u32 = 2;
pub(crate) const AUTH_RSAPUBLICKEY: u32 = 3;

mod adb_message_device;
mod adb_message_device_commands;
mod adb_message_transport;
mod adb_rsa_key;
mod adb_transport_message;
mod commands;
mod message_commands;
