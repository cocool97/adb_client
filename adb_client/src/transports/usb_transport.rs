use std::{sync::Arc, time::Duration};

use rusb::{
    constants::LIBUSB_CLASS_VENDOR_SPEC, Device, DeviceHandle, Direction, GlobalContext,
    TransferType,
};

use super::{ADBMessageTransport, ADBTransport};
use crate::{
    device::{ADBTransportMessage, ADBTransportMessageHeader, MessageCommand},
    Result, RustADBError,
};

#[derive(Debug)]
struct Endpoint {
    iface: u8,
    address: u8,
}

/// Transport running on USB
#[derive(Debug, Clone)]
pub struct USBTransport {
    device: Device<GlobalContext>,
    handle: Option<Arc<DeviceHandle<GlobalContext>>>,
}

impl USBTransport {
    /// Instantiate a new [`USBTransport`].
    /// Only the first device with given vendor_id and product_id is returned.
    pub fn new(vendor_id: u16, product_id: u16) -> Result<Self> {
        for device in rusb::devices()?.iter() {
            if let Ok(descriptor) = device.device_descriptor() {
                if descriptor.vendor_id() == vendor_id && descriptor.product_id() == product_id {
                    return Ok(Self::new_from_device(device));
                }
            }
        }

        Err(RustADBError::DeviceNotFound(format!(
            "cannot find USB device with vendor_id={} and product_id={}",
            vendor_id, product_id
        )))
    }

    /// Instantiate a new [`USBTransport`] from a [`rusb::Device`].
    ///
    /// Devices can be enumerated using [`rusb::devices()`] and then filtered out to get desired device.
    pub fn new_from_device(rusb_device: rusb::Device<GlobalContext>) -> Self {
        Self {
            device: rusb_device,
            handle: None,
        }
    }

    pub(crate) fn get_raw_connection(&self) -> Result<Arc<DeviceHandle<GlobalContext>>> {
        self.handle
            .as_ref()
            .ok_or(RustADBError::IOError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )))
            .cloned()
    }

    fn configure_endpoint(handle: &DeviceHandle<GlobalContext>, endpoint: &Endpoint) -> Result<()> {
        handle.claim_interface(endpoint.iface)?;
        Ok(())
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
        self.handle = Some(Arc::new(self.device.open()?));
        Ok(())
    }

    fn disconnect(&mut self) -> crate::Result<()> {
        let message = ADBTransportMessage::new(MessageCommand::Clse, 0, 0, "".into());
        self.write_message(message)
    }
}

impl ADBMessageTransport for USBTransport {
    fn write_message_with_timeout(
        &mut self,
        message: ADBTransportMessage,
        timeout: Duration,
    ) -> Result<()> {
        let endpoint = self.find_writable_endpoint()?;
        let handle = self.get_raw_connection()?;

        if let Ok(true) = handle.kernel_driver_active(endpoint.iface) {
            handle.detach_kernel_driver(endpoint.iface)?;
        }

        Self::configure_endpoint(&handle, &endpoint)?;

        let message_bytes = message.header().as_bytes()?;
        let mut total_written = 0;
        loop {
            total_written +=
                handle.write_bulk(endpoint.address, &message_bytes[total_written..], timeout)?;
            if total_written == message_bytes.len() {
                break;
            }
        }

        let payload = message.into_payload();
        if !payload.is_empty() {
            let mut total_written = 0;
            loop {
                total_written +=
                    handle.write_bulk(endpoint.address, &payload[total_written..], timeout)?;
                if total_written == payload.len() {
                    break;
                }
            }
        }

        Ok(())
    }

    fn read_message_with_timeout(&mut self, timeout: Duration) -> Result<ADBTransportMessage> {
        let endpoint = self.find_readable_endpoint()?;
        let handle = self.get_raw_connection()?;

        if let Ok(true) = handle.kernel_driver_active(endpoint.iface) {
            handle.detach_kernel_driver(endpoint.iface)?;
        }

        Self::configure_endpoint(&handle, &endpoint)?;

        let mut data = [0; 24];
        let mut total_read = 0;
        loop {
            total_read += handle.read_bulk(endpoint.address, &mut data[total_read..], timeout)?;
            if total_read == data.len() {
                break;
            }
        }

        let header = ADBTransportMessageHeader::try_from(data)?;

        log::trace!("received header {header:?}");

        if header.data_length() != 0 {
            let mut msg_data = vec![0_u8; header.data_length() as usize];
            let mut total_read = 0;
            loop {
                total_read +=
                    handle.read_bulk(endpoint.address, &mut msg_data[total_read..], timeout)?;
                if total_read == msg_data.capacity() {
                    break;
                }
            }

            let message = ADBTransportMessage::from_header_and_payload(header, msg_data);

            // Check message integrity
            if !message.check_message_integrity() {
                return Err(RustADBError::InvalidIntegrity(
                    ADBTransportMessageHeader::compute_crc32(message.payload()),
                    message.header().data_crc32(),
                ));
            }

            return Ok(message);
        }

        Ok(ADBTransportMessage::from_header_and_payload(header, vec![]))
    }
}
