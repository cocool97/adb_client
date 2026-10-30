mod list;
mod quit;
mod recv;
mod send;
mod stat;
mod sync_session;

pub use sync_session::ADBSyncSession;

use std::{
    io::{Read, Write},
    net::TcpStream,
};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{
    Result, RustADBError,
    models::{ADBCommand, ADBLocalCommand, SyncCommand},
    server_device::ADBServerDevice,
};

/// Size of the buffers used to transfer files. `DATA` chunks cannot exceed 64KiB (`SYNC_DATA_MAX` in adbd).
const MAX_BUFFER_SIZE: usize = 65535;

impl ADBServerDevice {
    /// Opens a sync session with this device, to run several file operations (stat, list, pull, push) on a
    /// single connection. See [`ADBSyncSession`].
    pub fn sync_session(&self) -> Result<ADBSyncSession> {
        let transport = self.connect_to_device()?;
        transport.send_adb_request(&ADBCommand::Local(ADBLocalCommand::Sync))?;

        Ok(ADBSyncSession {
            transport,
            session_closed: false,
        })
    }
}

/// Sends a sync request: its id, followed by the payload length and the payload.
fn send_sync_request(mut stream: &TcpStream, command: &SyncCommand, payload: &[u8]) -> Result<()> {
    let mut request = Vec::with_capacity(8 + payload.len());
    request.extend_from_slice(command.to_string().as_bytes());
    request.extend_from_slice(&u32::try_from(payload.len())?.to_le_bytes());
    request.extend_from_slice(payload);

    Ok(stream.write_all(&request)?)
}

/// Reads the 4 bytes id starting a sync response.
fn read_command_id(mut stream: &TcpStream) -> Result<[u8; 4]> {
    let mut id = [0_u8; 4];
    stream.read_exact(&mut id)?;
    Ok(id)
}

/// Builds the error matching an unexpected response id, reading the message of `FAIL` responses.
fn response_error(mut stream: &TcpStream, command_id: [u8; 4]) -> RustADBError {
    if &command_id != b"FAIL" {
        return RustADBError::UnknownResponseType(format!(
            "unexpected sync response {}",
            String::from_utf8_lossy(&command_id)
        ));
    }

    let mut read_message = || -> Result<String> {
        let length = stream.read_u32::<LittleEndian>()?;
        let mut message = vec![0; usize::try_from(length)?];
        stream.read_exact(&mut message)?;
        Ok(String::from_utf8_lossy(&message).into_owned())
    };

    match read_message() {
        Ok(message) => RustADBError::ADBRequestFailed(message),
        Err(e) => e,
    }
}
