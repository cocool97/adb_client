use std::time::Duration;

use crate::{
    Result,
    adb_transport::ADBTransport,
    message_devices::{
        adb_message_transport::ADBMessageTransport, adb_transport_message::ADBTransportMessage,
    },
};

/// Transport running on USB using `webusb` as a backend.
#[derive(Clone, Debug)]
pub struct WebUsbTransport {}

impl WebUsbTransport {
    /// Instantiate a new [`WebUsbTransport`].
    /// Only the first device with given vendor_id and product_id is returned.
    pub fn new() -> Self {
        WebUsbTransport {}
    }
}

impl ADBTransport for WebUsbTransport {
    fn connect(&mut self) -> crate::Result<()> {
        todo!()
    }

    fn disconnect(&mut self) -> crate::Result<()> {
        todo!()
    }
}

impl ADBMessageTransport for WebUsbTransport {
    fn read_message_with_timeout(
        &mut self,
        _read_timeout: Duration,
    ) -> Result<ADBTransportMessage> {
        todo!()
    }

    fn write_message_with_timeout(
        &mut self,
        message: ADBTransportMessage,
        write_timeout: std::time::Duration,
    ) -> Result<()> {
        todo!()
    }
}
