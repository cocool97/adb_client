use crate::{
    adb_transport::ADBTransport, message_devices::adb_message_transport::ADBMessageTransport,
    usb::ADBDeviceInfo, usb::USBTransport,
};

/// A transport implementation for WebUSB.
#[derive(Clone, Debug)]
pub struct WebUSBTransport {}

impl WebUSBTransport {
    /// Creates a new [`WebUSBTransport`] instance.
    pub fn new() -> Self {
        todo!()
    }
}

impl ADBTransport for WebUSBTransport {
    fn connect(&mut self) -> crate::Result<()> {
        todo!()
    }

    fn disconnect(&mut self) -> crate::Result<()> {
        todo!()
    }
}

impl ADBMessageTransport for WebUSBTransport {
    fn read_message_with_timeout(
        &mut self,
        read_timeout: std::time::Duration,
    ) -> crate::Result<crate::message_devices::adb_transport_message::ADBTransportMessage> {
        todo!()
    }

    fn write_message_with_timeout(
        &mut self,
        message: crate::message_devices::adb_transport_message::ADBTransportMessage,
        write_timeout: std::time::Duration,
    ) -> crate::Result<()> {
        todo!()
    }
}

impl USBTransport for WebUSBTransport {
    fn find_all_connected_adb_devices() -> crate::Result<Vec<ADBDeviceInfo>> {
        todo!()
    }

    fn vendor_id(&self) -> u16 {
        todo!()
    }

    fn product_id(&self) -> u16 {
        todo!()
    }
}
