mod adb_message_device;
mod adb_message_device_commands;
mod adb_tcp_device;
mod adb_transport_message;

#[cfg(feature = "usb")]
#[cfg_attr(docsrs, doc(cfg(feature = "usb")))]
mod adb_usb_device;
mod commands;
mod message_writer;
mod models;
mod shell_message_writer;

use adb_message_device::ADBMessageDevice;
pub use adb_tcp_device::ADBTcpDevice;
pub use adb_transport_message::{ADBTransportMessage, ADBTransportMessageHeader};

#[cfg(feature = "usb")]
#[cfg_attr(docsrs, doc(cfg(feature = "usb")))]
pub use adb_usb_device::{ADBUSBDevice, is_adb_device, search_adb_devices};

pub use message_writer::MessageWriter;
#[cfg(feature = "usb")]
#[cfg_attr(docsrs, doc(cfg(feature = "usb")))]
pub use models::ADBRsaKey;

pub use models::{MessageCommand, MessageSubcommand};
pub use shell_message_writer::ShellMessageWriter;
