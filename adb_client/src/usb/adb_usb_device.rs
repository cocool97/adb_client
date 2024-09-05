use crate::{ADBTransport, Result, USBTransport};
use super::ADBUsbMessageHeader;

/// Represent a device reached directly over USB
#[derive(Debug)]
pub struct ADBUSBDevice {
    transport: USBTransport,
}

impl ADBUSBDevice {
    /// Instantiate a new [ADBUSBDevice]
    pub fn new(vendor_id: u16, product_id: u16) -> Result<Self> {
        let transport = USBTransport::new(vendor_id, product_id);
        Ok(Self { transport })
    }

    /// TODO
    pub fn send_connect(&mut self) -> Result<()> {
        self.transport.connect()?;

        let message = ADBUsbMessageHeader::new(
            0x4e584e43,
            0x01000000,
            1048576,
            "host::pc-portable\0".into(),
        );

        self.transport.write(message)?;

        self.transport.read()?;

        Ok(())
    }
}
