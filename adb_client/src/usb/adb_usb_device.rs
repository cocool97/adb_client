use super::ADBUsbMessageHeader;
use crate::usb::constants::{CMD_CNXN, CONNECT_MAXDATA, CONNECT_PAYLOAD, CONNECT_VERSION};
use crate::{ADBTransport, Result, USBTransport};

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
            CMD_CNXN,
            CONNECT_VERSION,
            CONNECT_MAXDATA,
            CONNECT_PAYLOAD.into(),
        );

        self.transport.write(message)?;

        self.transport.read()?;

        Ok(())
    }
}
