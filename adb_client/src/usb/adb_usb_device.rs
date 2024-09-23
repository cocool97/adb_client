use std::path::PathBuf;

use super::ADBUsbMessage;
use crate::usb::adb_usb_message::{AUTH_RSAPUBLICKEY, AUTH_SIGNATURE, AUTH_TOKEN};
use crate::{usb::usb_commands::USBCommand, ADBTransport, Result, RustADBError, USBTransport};
use rsa::pkcs1::EncodeRsaPublicKey;
use rsa::signature::SignatureEncoding;
use rsa::signature::Signer;
use rsa::{pkcs1v15::SigningKey, pkcs8::DecodePrivateKey, RsaPrivateKey, RsaPublicKey};
use sha1::Sha1;

/// Represent a device reached directly over USB
#[derive(Debug)]
pub struct ADBUSBDevice {
    // Raw bytes from the PEM representation of the public key
    public_key: Vec<u8>,
    // Signing key derived from the private key for signing messages
    signing_key: SigningKey<Sha1>,
    transport: USBTransport,
}

fn read_adb_keypair(private_key: Option<PathBuf>) -> Option<RsaPrivateKey> {
    let private_key = private_key.or_else(|| {
        homedir::my_home()
            .ok()?
            .map(|home| home.join(".android").join("adbkey"))
    })?;

    let private_key = match std::fs::read_to_string(&private_key) {
        Ok(key) => RsaPrivateKey::from_pkcs8_pem(&key).expect("cannot load private key"),
        Err(e) => {
            log::warn!(
                "failed to read private key file: {}: {}",
                private_key.display(),
                e
            );
            return None;
        }
    };
    Some(private_key)
}

fn generate_keypair() -> RsaPrivateKey {
    log::info!("generating ephemeral RSA keypair");
    let mut rng = rand::thread_rng();
    let bits = 2048;

    RsaPrivateKey::new(&mut rng, bits).expect("failed to generate private key")
}

impl ADBUSBDevice {
    /// Instantiate a new [ADBUSBDevice]
    pub fn new(vendor_id: u16, product_id: u16, private_key: Option<PathBuf>) -> Result<Self> {
        let transport = USBTransport::new(vendor_id, product_id);
        let private_key = read_adb_keypair(private_key).unwrap_or_else(generate_keypair);
        let public_key_with_header = RsaPublicKey::from(&private_key)
            .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
            .expect("could not encode generated public key into pkcs1_pem");

        // Some devices seem to not like the ---BEGIN RSA PUBLIC KEY--- header
        let mut public_key = String::with_capacity(public_key_with_header.len());
        // ignore the header
        let mut lines = public_key_with_header.lines().skip(1);
        let mut prev = lines.next().unwrap();
        // the last value in current is never used (ignore the footer)
        while let Some(current) = lines.next() {
            public_key.push_str(prev);
            prev = current;
        }

        // println!("generated public key: {public_key}");
        public_key.push('\0');
        let public_key = public_key.into_bytes();
        let signing_key = SigningKey::<Sha1>::new(private_key);
        Ok(Self {
            public_key,
            signing_key,
            transport,
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

        let signed_payload = self.signing_key.try_sign(&auth_message.payload).unwrap();
        let b = signed_payload.to_vec();

        let message = ADBUsbMessage::new(USBCommand::Auth, AUTH_SIGNATURE, 0, b);
        self.transport.write_message(message)?;

        let received_response = self.transport.read_message()?;

        println!("response after auth signature: {:?}", &received_response);

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
