use std::fs::read_to_string;
use std::path::PathBuf;

use super::ADBUsbMessage;
use crate::usb::adb_usb_message::{AUTH_RSAPUBLICKEY, AUTH_SIGNATURE, AUTH_TOKEN};
use crate::{usb::usb_commands::USBCommand, ADBTransport, Result, RustADBError, USBTransport};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use rsa::pkcs1::EncodeRsaPublicKey;
use rsa::signature::SignatureEncoding;
use rsa::signature::Signer;
use rsa::{pkcs1v15::SigningKey, pkcs8::DecodePrivateKey, RsaPrivateKey, RsaPublicKey};
use sha1::Sha1;

/// Represent a device reached directly over USB
#[derive(Debug)]
pub struct ADBUSBDevice {
    // Raw bytes from the public key
    public_key: Vec<u8>,
    // Signing key derived from the private key for signing messages
    signing_key: SigningKey<Sha1>,
    transport: USBTransport,
}

fn read_adb_private_key(private_key_path: Option<PathBuf>) -> Option<RsaPrivateKey> {
    let private_key = private_key_path.or_else(|| {
        homedir::my_home()
            .ok()?
            .map(|home| home.join(".android").join("adbkey"))
    })?;

    read_to_string(&private_key)
        .map_err(RustADBError::from)
        .and_then(|pk| Ok(RsaPrivateKey::from_pkcs8_pem(&pk)?))
        .ok()
}

fn generate_keypair() -> Result<RsaPrivateKey> {
    log::info!("generating ephemeral RSA keypair");
    let mut rng = rand::thread_rng();
    Ok(RsaPrivateKey::new(&mut rng, 2048)?)
}

impl ADBUSBDevice {
    /// Instantiate a new [ADBUSBDevice]
    pub fn new(vendor_id: u16, product_id: u16, private_key_path: Option<PathBuf>) -> Result<Self> {
        let private_key = match read_adb_private_key(private_key_path) {
            Some(pk) => pk,
            None => generate_keypair()?,
        };

        let der_public_key = RsaPublicKey::from(&private_key).to_pkcs1_der()?;
        let mut public_key = BASE64_STANDARD.encode(der_public_key);
        public_key.push('\0');

        let signing_key = SigningKey::<Sha1>::new(private_key);
        Ok(Self {
            public_key: public_key.into_bytes(),
            signing_key,
            transport: USBTransport::new(vendor_id, product_id),
        })
    }

    /// Send initial connect
    pub fn send_connect(&mut self) -> Result<()> {
        self.transport.connect()?;

        // TO MAKE IT WORKING
        // WIRE USB DEVICE
        // IN NON ROOT RUN PROG

        let message = ADBUsbMessage::new(
            USBCommand::Cnxn,
            0x01000000,
            1048576,
            "host::pc-portable\0".into(),
        );

        self.transport.write_message(message)?;

        let message = self.transport.read_message()?;

        // At this point, we should have received either:
        // - an AUTH message with arg0 == 1
        // - a CNXN message
        let auth_message = match message.command {
            USBCommand::Auth if message.arg0 == AUTH_TOKEN => message,
            USBCommand::Auth if message.arg0 != AUTH_TOKEN => {
                return Err(RustADBError::ADBRequestFailed(
                    "Received AUTH message with type != 1".into(),
                ))
            }
            USBCommand::Cnxn => {
                log::info!("Successfully authenticated on device !");
                return Ok(());
            }
            _ => {
                return Err(RustADBError::ADBRequestFailed(format!(
                    "Wrong command received {}",
                    message.command
                )))
            }
        };

        let signed_payload = self.signing_key.try_sign(&auth_message.payload)?;
        let b = signed_payload.to_vec();

        let message = ADBUsbMessage::new(USBCommand::Auth, AUTH_SIGNATURE, 0, b);
        self.transport.write_message(message)?;

        let received_response = self.transport.read_message()?;

        if received_response.command == USBCommand::Cnxn {
            log::info!("Successfully authenticated on device !");
            return Ok(());
        }

        let message = ADBUsbMessage::new(
            USBCommand::Auth,
            AUTH_RSAPUBLICKEY,
            0,
            // TODO: Make the function accept a slice of u8
            // to avoid clone
            self.public_key.clone(),
        );

        self.transport.write_message(message)?;

        let response = self.transport.read_message()?;

        dbg!(response);

        Ok(())
    }
}

impl Drop for ADBUSBDevice {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.transport.disconnect();
    }
}
