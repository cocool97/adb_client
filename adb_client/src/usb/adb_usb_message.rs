use crate::RustADBError;

use super::usb_commands::USBCommand;

pub const AUTH_TOKEN: u32 = 1;
pub const AUTH_SIGNATURE: u32 = 2;
pub const AUTH_RSAPUBLICKEY: u32 = 3;

#[derive(Debug)]
pub struct ADBUsbMessage {
    pub command: USBCommand, /* command identifier constant      */
    pub arg0: u32,           /* first argument                   */
    pub arg1: u32,           /* second argument                  */
    pub data_length: u32,    /* length of payload (0 is allowed) */
    pub data_crc32: u32,     /* crc32 of data payload            */
    pub magic: u32,          /* command ^ 0xffffffff             */
    pub payload: Vec<u8>,
}

impl ADBUsbMessage {
    pub fn new(command: USBCommand, arg0: u32, arg1: u32, data: Vec<u8>) -> Self {
        let command_u32 = command.to_u32();
        Self {
            command,
            arg0,
            arg1,
            data_length: data.len() as u32,
            data_crc32: data.iter().map(|&x| x as u32).sum(),
            magic: command_u32 ^ 0xFFFFFFFF,
            payload: data,
        }
    }

    pub fn compute_checksum(&self) -> u32 {
        self.command.to_u32() ^ 0xFFFFFFFF
    }

    pub fn check_message_integrity(&self) -> bool {
        self.compute_checksum() == self.magic
    }

    pub fn to_bytes(&self) -> [u8; 24] {
        let mut result = [0u8; 24];
        let mut offset = 0;

        let command_bytes = self.command.to_u32().to_le_bytes();
        result[offset..offset + 4].copy_from_slice(&command_bytes);
        offset += 4;

        let arg0_bytes = self.arg0.to_le_bytes();
        result[offset..offset + 4].copy_from_slice(&arg0_bytes);
        offset += 4;

        let arg1_bytes = self.arg1.to_le_bytes();
        result[offset..offset + 4].copy_from_slice(&arg1_bytes);
        offset += 4;

        let data_length_bytes = self.data_length.to_le_bytes();
        result[offset..offset + 4].copy_from_slice(&data_length_bytes);
        offset += 4;

        let data_crc32_bytes = self.data_crc32.to_le_bytes();
        result[offset..offset + 4].copy_from_slice(&data_crc32_bytes);
        offset += 4;

        let magic_bytes = self.magic.to_le_bytes();
        result[offset..offset + 4].copy_from_slice(&magic_bytes);

        result
    }

    pub fn into_payload(self) -> Vec<u8> {
        self.payload
    }
}

impl TryFrom<[u8; 24]> for ADBUsbMessage {
    type Error = RustADBError;

    fn try_from(value: [u8; 24]) -> Result<Self, Self::Error> {
        let message = Self {
            command: USBCommand::try_from(&value[0..4])?,
            arg0: u32::from_le_bytes(value[4..8].try_into().unwrap()),
            arg1: u32::from_le_bytes(value[8..12].try_into().unwrap()),
            data_length: u32::from_le_bytes(value[12..16].try_into().unwrap()),
            data_crc32: u32::from_le_bytes(value[16..20].try_into().unwrap()),
            magic: u32::from_le_bytes(value[20..24].try_into().unwrap()),
            payload: vec![],
        };

        // Check checksum
        if !message.check_message_integrity() {
            return Err(RustADBError::InvalidCRC32(
                message.compute_checksum(),
                message.magic,
            ));
        }

        Ok(message)
    }
}
