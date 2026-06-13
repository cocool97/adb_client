/// USB-related definitions
#[cfg(feature = "usb")]
#[cfg_attr(docsrs, doc(cfg(feature = "usb")))]
pub mod usb;

/// Device reachable over TCP related definition
pub mod tcp;

pub(crate) mod adb_message_device;
mod adb_message_device_commands;
pub(crate) mod adb_message_transport;
/// ADB session management for open transport streams.
pub mod adb_session;
pub(crate) mod adb_transport_message;
mod commands;
pub(crate) mod message_commands;
mod models;
mod utils;

pub use adb_message_device::ADBMessageDevice;
pub use utils::BinaryDecodable;
