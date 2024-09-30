use serde::{Deserialize, Serialize};

use super::usb_commands::USBCommand;
use crate::RustADBError;
pub const AUTH_TOKEN: u32 = 1;
pub const AUTH_SIGNATURE: u32 = 2;
pub const AUTH_RSAPUBLICKEY: u32 = 3;

#[derive(Debug)]
pub struct ADBUsbMessage {
    pub header: AdbUsbMessageHeader,
    pub payload: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub(crate) struct AdbUsbMessageHeader {
    pub command: USBCommand, /* command identifier constant      */
    pub arg0: u32,           /* first argument                   */
    pub arg1: u32,           /* second argument                  */
    pub data_length: u32,    /* length of payload (0 is allowed) */
    pub data_crc32: u32,     /* crc32 of data payload            */
    pub magic: u32,          /* command ^ 0xffffffff             */
}

impl AdbUsbMessageHeader {
    pub fn compute_checksum(&self) -> u32 {
        self.command as u32 ^ 0xFFFFFFFF
    }

    pub fn check_message_integrity(&self) -> bool {
        self.compute_checksum() == self.magic
    }
}

impl ADBUsbMessage {
    pub fn new(command: USBCommand, arg0: u32, arg1: u32, data: Vec<u8>) -> Self {
        Self {
            header: AdbUsbMessageHeader {
                command,
                arg0,
                arg1,
                data_length: data.len() as u32,
                data_crc32: data.iter().map(|&x| x as u32).sum(),
                magic: command as u32 ^ 0xFFFFFFFF,
            },
            payload: data,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // safety: the header struct is public only to crate modules
        // we ensure that we control its creation and modifications
        bincode::serialize(&self.header).unwrap()
    }

    pub fn into_payload(self) -> Vec<u8> {
        self.payload
    }
}

impl TryFrom<[u8; 24]> for ADBUsbMessage {
    type Error = RustADBError;

    fn try_from(value: [u8; 24]) -> Result<Self, Self::Error> {
        // TODO: add variant in Error enum to remove this unwrap
        let header: AdbUsbMessageHeader =
            bincode::deserialize(&value).map_err(|_e| RustADBError::ConversionError)?;

        // Check checksum
        if !header.check_message_integrity() {
            return Err(RustADBError::InvalidCRC32(
                header.compute_checksum(),
                header.magic,
            ));
        }

        Ok(Self {
            header,
            payload: vec![],
        })
    }
}
