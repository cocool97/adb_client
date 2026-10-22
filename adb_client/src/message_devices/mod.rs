/// USB-related definitions
#[cfg(all(feature = "usb", not(target_arch = "wasm32")))]
#[cfg_attr(docsrs, doc(cfg(feature = "usb")))]
pub mod usb;

/// Device reachable over TCP related definition
#[cfg(not(target_arch = "wasm32"))]
pub mod tcp;

#[cfg(not(target_arch = "wasm32"))]
mod adb_message_device;
#[cfg(not(target_arch = "wasm32"))]
mod adb_message_device_commands;
#[cfg(not(target_arch = "wasm32"))]
mod adb_message_transport;
#[cfg(not(target_arch = "wasm32"))]
mod adb_session;
mod adb_transport_message;
#[cfg(not(target_arch = "wasm32"))]
mod commands;
mod message_commands;
mod models;
mod utils;

pub use utils::BinaryDecodable;

/// Asynchronous browser USB connections and in-memory host keys.
#[cfg(feature = "webusb")]
#[cfg_attr(docsrs, doc(cfg(feature = "webusb")))]
pub mod webusb;
