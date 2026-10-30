use std::io::Read;

use super::{ADBSyncSession, read_command_id, response_error, send_sync_request};
use crate::{
    Result,
    models::{AdbStatResponse, SyncCommand},
};

impl ADBSyncSession {
    /// Stat file given as path on the device.
    pub fn stat(&mut self, path: &str) -> Result<AdbStatResponse> {
        self.run(|mut stream| {
            send_sync_request(stream, &SyncCommand::Stat, path.as_bytes())?;

            match read_command_id(stream)? {
                id if &id == b"STAT" => {
                    let mut data = [0_u8; 12];
                    stream.read_exact(&mut data)?;
                    Ok(data.into())
                }
                id => Err(response_error(stream, id)),
            }
        })
    }
}
