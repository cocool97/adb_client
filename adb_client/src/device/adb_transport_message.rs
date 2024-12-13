use serde::{Deserialize, Serialize};

use crate::{Result, RustADBError};

use super::models::MessageCommand;

pub const AUTH_TOKEN: u32 = 1;
pub const AUTH_SIGNATURE: u32 = 2;
pub const AUTH_RSAPUBLICKEY: u32 = 3;

#[derive(Debug)]
pub struct ADBTransportMessage {
    header: ADBTransportMessageHeader,
    payload: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ADBTransportMessageHeader {
    command: MessageCommand, /* command identifier constant      */
    arg0: u32,               /* first argument                   */
    arg1: u32,               /* second argument                  */
    data_length: u32,        /* length of payload (0 is allowed) */
    data_crc32: u32,         /* crc32 of data payload            */
    magic: u32,              /* command ^ 0xffffffff             */
}

impl ADBTransportMessageHeader {
    pub fn new(command: MessageCommand, arg0: u32, arg1: u32, data: &[u8]) -> Self {
        Self {
            command,
            arg0,
            arg1,
            data_length: data.len() as u32,
            data_crc32: Self::compute_crc32(data),
            magic: Self::compute_magic(command),
        }
    }

    pub fn command(&self) -> MessageCommand {
        self.command
    }

    pub fn arg0(&self) -> u32 {
        self.arg0
    }

    pub fn arg1(&self) -> u32 {
        self.arg1
    }

    pub fn data_length(&self) -> u32 {
        self.data_length
    }

    pub fn data_crc32(&self) -> u32 {
        self.data_crc32
    }

    pub(crate) fn compute_crc32(data: &[u8]) -> u32 {
        data.iter().map(|&x| x as u32).sum()
    }

    fn compute_magic(command: MessageCommand) -> u32 {
        let command_u32 = command as u32;
        command_u32 ^ 0xFFFFFFFF
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(&self).map_err(|_e| RustADBError::ConversionError)
    }
}

impl ADBTransportMessage {
    pub fn new(command: MessageCommand, arg0: u32, arg1: u32, data: &[u8]) -> Self {
        Self {
            header: ADBTransportMessageHeader::new(command, arg0, arg1, data),
            payload: data.to_vec(),
        }
    }

    pub fn from_header_and_payload(header: ADBTransportMessageHeader, payload: Vec<u8>) -> Self {
        Self { header, payload }
    }

    pub fn check_message_integrity(&self) -> bool {
        ADBTransportMessageHeader::compute_magic(self.header.command) == self.header.magic
            && ADBTransportMessageHeader::compute_crc32(&self.payload) == self.header.data_crc32
    }

    pub fn assert_command(&self, expected_command: MessageCommand) -> Result<()> {
        let our_command = self.header().command();
        if expected_command == our_command {
            return Ok(());
        }

        Err(RustADBError::WrongResponseReceived(
            our_command.to_string(),
            expected_command.to_string(),
        ))
    }

    pub fn header(&self) -> &ADBTransportMessageHeader {
        &self.header
    }

    pub fn payload(&self) -> &Vec<u8> {
        &self.payload
    }

    pub fn into_payload(self) -> Vec<u8> {
        self.payload
    }
}

impl TryFrom<[u8; 24]> for ADBTransportMessageHeader {
    type Error = RustADBError;

    fn try_from(value: [u8; 24]) -> Result<Self> {
        bincode::deserialize(&value).map_err(|_e| RustADBError::ConversionError)
    }
}
