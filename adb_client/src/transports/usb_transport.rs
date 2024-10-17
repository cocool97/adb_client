use std::time::Duration;

use rusb::{
    constants::LIBUSB_CLASS_VENDOR_SPEC, DeviceHandle, Direction, GlobalContext, TransferType,
};

use super::ADBTransport;
use crate::{
    usb::{ADBUsbMessage, USBCommand},
    Result, RustADBError,
};

#[derive(Debug)]
struct Endpoint {
    iface: u8,
    address: u8,
}

const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(2);
const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_secs(2);

/// Transport running on USB
#[derive(Debug)]
pub struct USBTransport {
    vendor_id: u16,
    product_id: u16,
    handle: Option<DeviceHandle<GlobalContext>>,
}

impl USBTransport {
    /// Instantiate a new [USBTransport]
    pub fn new(vendor_id: u16, product_id: u16) -> Self {
        Self {
            handle: None,
            vendor_id,
            product_id,
        }
    }

    pub(crate) fn get_raw_connection(&self) -> Result<&DeviceHandle<GlobalContext>> {
        self.handle
            .as_ref()
            .ok_or(RustADBError::IOError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )))
    }

    fn configure_endpoint(handle: &DeviceHandle<GlobalContext>, endpoint: &Endpoint) -> Result<()> {
        handle.claim_interface(endpoint.iface)?;
        Ok(())
    }

    /// Write data to underlying connection, with default timeout
    pub(crate) fn write_message(&mut self, message: ADBUsbMessage) -> Result<()> {
        self.write_message_with_timeout(message, DEFAULT_WRITE_TIMEOUT)
    }

    /// Write data to underlying connection
    pub(crate) fn write_message_with_timeout(
        &mut self,
        message: ADBUsbMessage,
        timeout: Duration,
    ) -> Result<()> {
        let endpoint = self.find_writable_endpoint()?;
        let handle = self.get_raw_connection()?;

        if let Ok(true) = handle.kernel_driver_active(endpoint.iface) {
            handle.detach_kernel_driver(endpoint.iface)?;
        }

        Self::configure_endpoint(handle, &endpoint)?;

        // TODO: loop
        let message_bytes = message.to_bytes()?;
        let written = handle.write_bulk(endpoint.address, &message_bytes, timeout)?;

        // TODO: loop
        let payload = message.into_payload();
        let written = handle.write_bulk(endpoint.address, &payload, timeout)?;

        Ok(())
    }

    /// Read data from underlying connection with default timeout
    pub(crate) fn read_message(&mut self) -> Result<ADBUsbMessage> {
        self.read_message_with_timeout(DEFAULT_READ_TIMEOUT)
    }

    /// Read data from underlying connection with given timeout
    pub(crate) fn read_message_with_timeout(&mut self, timeout: Duration) -> Result<ADBUsbMessage> {
        let endpoint = self.find_readable_endpoint()?;
        let handle = self.get_raw_connection()?;

        if let Ok(true) = handle.kernel_driver_active(endpoint.iface) {
            handle.detach_kernel_driver(endpoint.iface)?;
        }

        Self::configure_endpoint(handle, &endpoint)?;

        let mut data = [0; 24];
        // TODO: loop
        let read = handle.read_bulk(endpoint.address, &mut data, timeout)?;

        let mut message = ADBUsbMessage::try_from(data)?;

        if message.data_length() != 0 {
            let mut msg_data = vec![0_u8; message.data_length() as usize];
            // TODO: loop
            let read = handle.read_bulk(endpoint.address, &mut msg_data, timeout)?;
            message.with_payload(msg_data);
        }

        log::trace!("read {message:?}");

        Ok(message)
    }

    fn find_readable_endpoint(&self) -> Result<Endpoint> {
        let handle = self.get_raw_connection()?;
        for n in 0..handle.device().device_descriptor()?.num_configurations() {
            let config_desc = match handle.device().config_descriptor(n) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for interface in config_desc.interfaces() {
                for interface_desc in interface.descriptors() {
                    for endpoint_desc in interface_desc.endpoint_descriptors() {
                        if endpoint_desc.direction() == Direction::In
                            && endpoint_desc.transfer_type() == TransferType::Bulk
                            && interface_desc.class_code() == LIBUSB_CLASS_VENDOR_SPEC
                            && interface_desc.sub_class_code() == 0x42
                            && interface_desc.protocol_code() == 0x01
                        {
                            return Ok(Endpoint {
                                iface: interface_desc.interface_number(),
                                address: endpoint_desc.address(),
                            });
                        }
                    }
                }
            }
        }

        Err(RustADBError::USBNoDescriptorFound)
    }

    fn find_writable_endpoint(&self) -> Result<Endpoint> {
        let handle = self.get_raw_connection()?;
        for n in 0..handle.device().device_descriptor()?.num_configurations() {
            let config_desc = match handle.device().config_descriptor(n) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for interface in config_desc.interfaces() {
                for interface_desc in interface.descriptors() {
                    for endpoint_desc in interface_desc.endpoint_descriptors() {
                        if endpoint_desc.direction() == Direction::Out
                            && endpoint_desc.transfer_type() == TransferType::Bulk
                            && interface_desc.class_code() == LIBUSB_CLASS_VENDOR_SPEC
                            && interface_desc.sub_class_code() == 0x42
                            && interface_desc.protocol_code() == 0x01
                        {
                            return Ok(Endpoint {
                                iface: interface_desc.interface_number(),
                                address: endpoint_desc.address(),
                            });
                        }
                    }
                }
            }
        }

        Err(RustADBError::USBNoDescriptorFound)
    }
}

impl ADBTransport for USBTransport {
    fn connect(&mut self) -> crate::Result<()> {
        // Remove in production
        let handle = rusb::open_device_with_vid_pid(self.vendor_id, self.product_id).ok_or(
            RustADBError::USBDeviceNotFound(self.vendor_id, self.product_id),
        )?;

        self.handle = Some(handle);

        Ok(())
    }

    fn disconnect(&mut self) -> crate::Result<()> {
        let message = ADBUsbMessage::new(USBCommand::Clse, 0, 0, "".into());
        self.write_message(message)
    }
}
