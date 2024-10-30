use byteorder::ReadBytesExt;
use rand::Rng;
use retry::{delay::Fixed, retry};
use rusb::Device;
use rusb::DeviceDescriptor;
use rusb::UsbContext;
use std::fs::read_to_string;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use byteorder::LittleEndian;

use super::{ADBRsaKey, ADBUsbMessage};
use crate::constants::BUFFER_SIZE;
use crate::models::AdbStatResponse;
use crate::usb::adb_usb_message::{AUTH_RSAPUBLICKEY, AUTH_SIGNATURE, AUTH_TOKEN};
use crate::{
    usb::usb_commands::{USBCommand, USBSubcommand},
    ADBTransport, Result, RustADBError, USBTransport,
};

/// Represent a device reached directly over USB
#[derive(Debug)]
pub struct ADBUSBDevice {
    private_key: ADBRsaKey,
    pub(crate) transport: USBTransport,
}

fn read_adb_private_key<P: AsRef<Path>>(private_key_path: P) -> Result<Option<ADBRsaKey>> {
    read_to_string(private_key_path.as_ref())
        .map_err(RustADBError::from)
        .map(|pk| match ADBRsaKey::from_pkcs8(&pk) {
            Ok(pk) => Some(pk),
            Err(e) => {
                log::error!("Error while create RSA private key: {e}");
                None
            }
        })
}
/// Search for adb devices with known interface class and subclass values
fn search_adb_devices() -> Option<(u16, u16)> {
    for device in rusb::devices().unwrap().iter() {
        let Ok(des) = device.device_descriptor() else {
            continue;
        };
        if is_adb_device(&device, &des) {
            return Some((des.vendor_id(), des.product_id()));
        }
    }
    None
}

fn is_adb_device<T: UsbContext>(device: &Device<T>, des: &DeviceDescriptor) -> bool {
    for n in 0..des.num_configurations() {
        let Ok(config_des) = device.config_descriptor(n) else {
            continue;
        };
        for interface in config_des.interfaces() {
            for interface_des in interface.descriptors() {
                let proto = interface_des.protocol_code();
                let class = interface_des.class_code();
                let subcl = interface_des.sub_class_code();
                if proto == 1 && ((class == 0xff && subcl == 0x42) || (class == 0xdc && subcl == 2))
                {
                    return true;
                }
            }
        }
    }
    false
}

impl ADBUSBDevice {
    /// Instantiate a new [ADBUSBDevice]
    pub fn new(vendor_id: u16, product_id: u16) -> Result<Self> {
        let private_key_path = homedir::my_home()
            .ok()
            .flatten()
            .map(|home| home.join(".android").join("adbkey"))
            .ok_or(RustADBError::NoHomeDirectory)?;

        let private_key = match read_adb_private_key(private_key_path)? {
            Some(pk) => pk,
            None => ADBRsaKey::random_with_size(2048)?,
        };

        let mut s = Self {
            private_key,
            transport: USBTransport::new(vendor_id, product_id),
        };

        s.connect()?;

        Ok(s)
    }

    /// autodetect connected ADB devices and establish a connection with the
    /// first device found
    pub fn autodetect() -> Result<Self> {
        retry(Fixed::from_millis(3000).take(5), || {
            let Some((vid, pid)) = search_adb_devices() else {
                return Err(RustADBError::ADBRequestFailed(format!(
                    "no USB devices found that match the signature of an ADB device"
                )));
            };
            log::trace!("Trying to connect to ({vid}, {pid})");
            ADBUSBDevice::new(vid, pid)
        })
        .map_err(|e| {
            RustADBError::ADBRequestFailed(format!("the device took too long to respond: {e}"))
        })
    }

    /// autodetect connected ADB devices and establish a connection with the
    /// first device found using a custom private key path
    pub fn autodetect_with_custom_private_key(private_key_path: PathBuf) -> Result<Self> {
        retry(Fixed::from_millis(3000).take(5), || {
            let Some((vid, pid)) = search_adb_devices() else {
                return Err(RustADBError::ADBRequestFailed(format!(
                    "no USB devices found that match the signature of an ADB device"
                )));
            };
            log::trace!("Trying to connect to ({vid}, {pid})");
            ADBUSBDevice::new_with_custom_private_key(vid, pid, private_key_path.clone())
        })
        .map_err(|e| {
            RustADBError::ADBRequestFailed(format!("the device took too long to respond: {e}"))
        })
    }

    /// Instantiate a new [ADBUSBDevice] using a custom private key path
    pub fn new_with_custom_private_key(
        vendor_id: u16,
        product_id: u16,
        private_key_path: PathBuf,
    ) -> Result<Self> {
        let private_key = match read_adb_private_key(private_key_path)? {
            Some(pk) => pk,
            None => ADBRsaKey::random_with_size(2048)?,
        };

        let mut s = Self {
            private_key,
            transport: USBTransport::new(vendor_id, product_id),
        };

        s.connect()?;

        Ok(s)
    }

    /// Send initial connect
    pub fn connect(&mut self) -> Result<()> {
        self.transport.connect()?;

        let message = ADBUsbMessage::new(
            USBCommand::Cnxn,
            0x01000000,
            1048576,
            format!("host::{}\0", env!("CARGO_PKG_NAME"))
                .as_bytes()
                .to_vec(),
        );

        self.transport.write_message(message)?;

        let message = self.transport.read_message()?;

        // At this point, we should have received either:
        // - an AUTH message with arg0 == 1
        // - a CNXN message
        let auth_message = match message.header().command() {
            USBCommand::Auth if message.header().arg0() == AUTH_TOKEN => message,
            USBCommand::Auth if message.header().arg0() != AUTH_TOKEN => {
                return Err(RustADBError::ADBRequestFailed(
                    "Received AUTH message with type != 1".into(),
                ))
            }
            c => {
                return Err(RustADBError::ADBRequestFailed(format!(
                    "Wrong command received {}",
                    c
                )))
            }
        };

        let sign = self.private_key.sign(auth_message.into_payload())?;

        let message = ADBUsbMessage::new(USBCommand::Auth, AUTH_SIGNATURE, 0, sign);

        self.transport.write_message(message)?;

        let received_response = self.transport.read_message()?;

        if received_response.header().command() == USBCommand::Cnxn {
            log::info!(
                "Authentication OK, device info {}",
                String::from_utf8(received_response.into_payload())?
            );
            return Ok(());
        }

        let mut pubkey = self.private_key.encoded_public_key()?.into_bytes();
        pubkey.push(b'\0');

        let message = ADBUsbMessage::new(USBCommand::Auth, AUTH_RSAPUBLICKEY, 0, pubkey);

        self.transport.write_message(message)?;

        let response = self
            .transport
            .read_message_with_timeout(Duration::from_secs(10))?;

        match response.header().command() {
            USBCommand::Cnxn => log::info!(
                "Authentication OK, device info {}",
                String::from_utf8(response.into_payload())?
            ),
            _ => {
                return Err(RustADBError::ADBRequestFailed(format!(
                    "wrong response {}",
                    response.header().command()
                )))
            }
        }

        Ok(())
    }
    /// Receive a message and acknowledge it by replying with an `OKAY` command
    pub(crate) fn recv_and_reply_okay(
        &mut self,
        local_id: u32,
        remote_id: u32,
    ) -> Result<ADBUsbMessage> {
        let message = self.transport.read_message()?;
        self.transport.write_message(ADBUsbMessage::new(
            USBCommand::Okay,
            local_id,
            remote_id,
            "".into(),
        ))?;
        Ok(message)
    }

    /// Expect a message with an `OKAY` command after sending a message.
    pub(crate) fn send_and_expect_okay(&mut self, message: ADBUsbMessage) -> Result<ADBUsbMessage> {
        self.transport.write_message(message)?;
        let message = self.transport.read_message()?;
        let received_command = message.header().command();
        if received_command != USBCommand::Okay {
            return Err(RustADBError::ADBRequestFailed(format!(
                "expected command OKAY after message, got {}",
                received_command
            )));
        }
        Ok(message)
    }

    pub(crate) fn recv_file<W: std::io::Write>(
        &mut self,
        local_id: u32,
        remote_id: u32,
        mut output: W,
    ) -> std::result::Result<(), RustADBError> {
        let mut len: Option<u64> = None;
        loop {
            let payload = self
                .recv_and_reply_okay(local_id, remote_id)?
                .into_payload();
            let mut rdr = Cursor::new(&payload);
            while rdr.position() != payload.len() as u64 {
                match len.take() {
                    Some(0) | None => {
                        rdr.seek_relative(4)?;
                        len.replace(rdr.read_u32::<LittleEndian>()? as u64);
                    }
                    Some(length) => {
                        log::debug!("len = {length}");
                        let remaining_bytes = payload.len() as u64 - rdr.position();
                        log::debug!(
                            "payload length {} - reader_position {} = {remaining_bytes}",
                            payload.len(),
                            rdr.position()
                        );
                        if length < remaining_bytes {
                            let read = std::io::copy(&mut rdr.by_ref().take(length), &mut output)?;
                            log::debug!(
                                "expected to read {length} bytes, actually read {read} bytes"
                            );
                        } else {
                            let read = std::io::copy(&mut rdr.take(remaining_bytes), &mut output)?;
                            len.replace(length - remaining_bytes);
                            log::debug!("expected to read {remaining_bytes} bytes, actually read {read} bytes");
                            // this payload is exhausted
                            break;
                        }
                    }
                }
            }
            if Cursor::new(&payload[(payload.len() - 8)..(payload.len() - 4)])
                .read_u32::<LittleEndian>()?
                == USBSubcommand::Done as u32
            {
                break;
            }
        }
        Ok(())
    }

    pub(crate) fn push_file<R: std::io::Read>(
        &mut self,
        local_id: u32,
        remote_id: u32,
        mut reader: R,
    ) -> std::result::Result<(), RustADBError> {
        let mut buffer = [0; BUFFER_SIZE];
        let amount_read = reader.read(&mut buffer)?;
        let subcommand_data = USBSubcommand::Data.with_arg(amount_read as u32);

        let mut serialized_message =
            bincode::serialize(&subcommand_data).map_err(|_e| RustADBError::ConversionError)?;
        serialized_message.append(&mut buffer[..amount_read].to_vec());

        let message =
            ADBUsbMessage::new(USBCommand::Write, local_id, remote_id, serialized_message);

        self.send_and_expect_okay(message)?;

        loop {
            let mut buffer = [0; BUFFER_SIZE];

            match reader.read(&mut buffer) {
                Ok(0) => {
                    // Currently file mtime is not forwarded
                    let subcommand_data = USBSubcommand::Done.with_arg(0);

                    let serialized_message = bincode::serialize(&subcommand_data)
                        .map_err(|_e| RustADBError::ConversionError)?;

                    let message = ADBUsbMessage::new(
                        USBCommand::Write,
                        local_id,
                        remote_id,
                        serialized_message,
                    );

                    self.send_and_expect_okay(message)?;

                    // Command should end with a Write => Okay
                    let received = self.transport.read_message()?;
                    match received.header().command() {
                        USBCommand::Write => return Ok(()),
                        c => {
                            return Err(RustADBError::ADBRequestFailed(format!(
                                "Wrong command received {}",
                                c
                            )))
                        }
                    }
                }
                Ok(size) => {
                    let subcommand_data = USBSubcommand::Data.with_arg(size as u32);

                    let mut serialized_message = bincode::serialize(&subcommand_data)
                        .map_err(|_e| RustADBError::ConversionError)?;
                    serialized_message.append(&mut buffer[..size].to_vec());

                    let message = ADBUsbMessage::new(
                        USBCommand::Write,
                        local_id,
                        remote_id,
                        serialized_message,
                    );

                    self.send_and_expect_okay(message)?;
                }
                Err(e) => {
                    return Err(RustADBError::IOError(e));
                }
            }
        }
    }

    pub(crate) fn begin_synchronization(&mut self) -> Result<(u32, u32)> {
        let sync_directive = "sync:\0";

        let mut rng = rand::thread_rng();
        let message = ADBUsbMessage::new(
            USBCommand::Open,
            rng.gen(), /* Our 'local-id' */
            0,
            sync_directive.into(),
        );
        let message = self.send_and_expect_okay(message)?;
        let local_id = message.header().arg1();
        let remote_id = message.header().arg0();
        Ok((local_id, remote_id))
    }

    pub(crate) fn stat_with_explicit_ids(
        &mut self,
        remote_path: &str,
        local_id: u32,
        remote_id: u32,
    ) -> Result<AdbStatResponse> {
        let stat_buffer = USBSubcommand::Stat.with_arg(remote_path.len() as u32);
        let message = ADBUsbMessage::new(
            USBCommand::Write,
            local_id,
            remote_id,
            bincode::serialize(&stat_buffer).map_err(|_e| RustADBError::ConversionError)?,
        );
        self.send_and_expect_okay(message)?;
        self.send_and_expect_okay(ADBUsbMessage::new(
            USBCommand::Write,
            local_id,
            remote_id,
            remote_path.into(),
        ))?;
        let response = self.transport.read_message()?;
        // Skip first 4 bytes as this is the literal "STAT".
        // Interesting part starts right after
        bincode::deserialize(&response.into_payload()[4..])
            .map_err(|_e| RustADBError::ConversionError)
    }

    pub(crate) fn end_transaction(&mut self, local_id: u32, remote_id: u32) -> Result<()> {
        let quit_buffer = USBSubcommand::Quit.with_arg(0u32);
        self.send_and_expect_okay(ADBUsbMessage::new(
            USBCommand::Write,
            local_id,
            remote_id,
            bincode::serialize(&quit_buffer).map_err(|_e| RustADBError::ConversionError)?,
        ))?;
        let _discard_close = self.transport.read_message()?;
        Ok(())
    }
}

impl Drop for ADBUSBDevice {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.transport.disconnect();
    }
}
