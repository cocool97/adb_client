mod tcp_emulator_transport;
mod tcp_server_transport;
mod tcp_transport;
mod traits;

#[cfg(feature = "usb")]
#[cfg_attr(docsrs, doc(cfg(feature = "usb")))]
mod usb_transport;

pub use tcp_emulator_transport::TCPEmulatorTransport;
pub use tcp_server_transport::TCPServerTransport;
pub use tcp_transport::TcpTransport;
pub use traits::{ADBMessageTransport, ADBTransport};

#[cfg(feature = "usb")]
#[cfg_attr(docsrs, doc(cfg(feature = "usb")))]
pub use usb_transport::USBTransport;
