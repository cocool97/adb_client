use serde::{Deserialize, Serialize};

use super::usb_commands::USBCommand;
use crate::RustADBError;

pub const AUTH_TOKEN: u32 = 1;
pub const AUTH_SIGNATURE: u32 = 2;
pub const AUTH_RSAPUBLICKEY: u32 = 3;

#[derive(Debug)]
pub struct ADBUsbMessage {
    header: ADBUsbMessageHeader,
    payload: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ADBUsbMessageHeader {
    command: USBCommand, /* command identifier constant      */
    arg0: u32,           /* first argument                   */
    arg1: u32,           /* second argument                  */
    data_length: u32,    /* length of payload (0 is allowed) */
    data_crc32: u32,     /* crc32 of data payload            */
    magic: u32,          /* command ^ 0xffffffff             */
}

impl ADBUsbMessage {
    pub fn new(command: USBCommand, arg0: u32, arg1: u32, data: Vec<u8>) -> Self {
        let command_u32 = command as u32;
        Self {
            header: ADBUsbMessageHeader {
                command,
                arg0,
                arg1,
                data_length: data.len() as u32,
                data_crc32: data.iter().map(|&x| x as u32).sum(),
                magic: command_u32 ^ 0xFFFFFFFF,
            },
            payload: data,
        }
    }

    pub fn compute_checksum(&self) -> u32 {
        self.header.command as u32 ^ 0xFFFFFFFF
    }

    pub fn check_message_integrity(&self) -> bool {
        self.compute_checksum() == self.header.magic
    }

    pub fn command(&self) -> USBCommand {
        self.header.command
    }

    pub fn arg0(&self) -> u32 {
        self.header.arg0
    }

    pub fn data_length(&self) -> u32 {
        self.header.data_length
    }

    pub fn into_payload(self) -> Vec<u8> {
        self.payload
    }

    pub fn with_payload(&mut self, payload: Vec<u8>) {
        self.payload = payload;
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, RustADBError> {
        bincode::serialize(&self.header).map_err(|_e| RustADBError::ConversionError)
    }
}

impl TryFrom<[u8; 24]> for ADBUsbMessage {
    type Error = RustADBError;

    fn try_from(value: [u8; 24]) -> Result<Self, Self::Error> {
        let header = bincode::deserialize(&value).map_err(|_e| RustADBError::ConversionError)?;
        let message = ADBUsbMessage {
            header,
            payload: vec![],
        };

        // Check checksum
        if !message.check_message_integrity() {
            return Err(RustADBError::InvalidCRC32(
                message.compute_checksum(),
                message.header.magic,
            ));
        }

        Ok(message)
    }
}
