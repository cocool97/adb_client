use std::fs::read_to_string;
use std::path::PathBuf;
use std::time::Duration;

use super::{ADBRsaKey, ADBUsbMessage};
use crate::usb::adb_usb_message::{AUTH_RSAPUBLICKEY, AUTH_SIGNATURE, AUTH_TOKEN};
use crate::{usb::usb_commands::USBCommand, ADBTransport, Result, RustADBError, USBTransport};

/// Represent a device reached directly over USB
#[derive(Debug)]
pub struct ADBUSBDevice {
    private_key: ADBRsaKey,
    pub(crate) transport: USBTransport,
}

fn read_adb_private_key(private_key_path: Option<PathBuf>) -> Option<ADBRsaKey> {
    let private_key = private_key_path.or_else(|| {
        homedir::my_home()
            .ok()?
            .map(|home| home.join(".android").join("adbkey"))
    })?;

    read_to_string(&private_key)
        .map_err(RustADBError::from)
        .map(|pk| ADBRsaKey::from_pkcs8(&pk).unwrap())
        .ok()
}

impl ADBUSBDevice {
    /// Instantiate a new [ADBUSBDevice]
    pub fn new(vendor_id: u16, product_id: u16, private_key_path: Option<PathBuf>) -> Result<Self> {
        let private_key = match read_adb_private_key(private_key_path) {
            Some(pk) => pk,
            None => unimplemented!(),
        };

        Ok(Self {
            private_key,
            transport: USBTransport::new(vendor_id, product_id),
        })
    }

    /// Send initial connect
    pub fn send_connect(&mut self) -> Result<()> {
        self.transport.connect()?;

        let message = ADBUsbMessage::new(
            USBCommand::Cnxn,
            0x01000000,
            1048576,
            "host::pc-portable\0".as_bytes().to_vec(),
        );

        self.transport.write_message(message)?;

        let message = self.transport.read_message()?;

        // At this point, we should have received either:
        // - an AUTH message with arg0 == 1
        // - a CNXN message
        let auth_message = match message.command() {
            USBCommand::Auth if message.arg0() == AUTH_TOKEN => message,
            USBCommand::Auth if message.arg0() != AUTH_TOKEN => {
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

        let sign = self.private_key.sign(auth_message.into_payload()).unwrap();

        let message = ADBUsbMessage::new(USBCommand::Auth, AUTH_SIGNATURE, 0, sign);

        self.transport.write_message(message)?;

        let received_response = self.transport.read_message()?;

        if received_response.command() == USBCommand::Cnxn {
            log::info!("Successfully authenticated on device !");
            return Ok(());
        }

        let mut pubkey = self.private_key.encoded_public_key().unwrap().into_bytes();
        pubkey.push(b'\0');

        let message = ADBUsbMessage::new(USBCommand::Auth, AUTH_RSAPUBLICKEY, 0, pubkey);

        self.transport.write_message(message)?;

        let response = self
            .transport
            .read_message_with_timeout(Duration::from_secs(10))?;

        match response.command() {
            USBCommand::Cnxn => log::info!(
                "Authentication OK, device info {}",
                String::from_utf8(response.into_payload().to_vec()).unwrap()
            ),
            _ => {
                return Err(RustADBError::ADBRequestFailed(format!(
                    "wrong response {}",
                    response.command()
                )))
            }
        }

        Ok(())
    }
}

impl Drop for ADBUSBDevice {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.transport.disconnect();
    }
}
