use std::{
    fmt::Display,
    io::{Read, Write},
    time::{Duration, UNIX_EPOCH},
};

use byteorder::{ByteOrder, LittleEndian};
use chrono::{DateTime, Utc};

use crate::{
    models::{AdbServerCommand, SyncCommand},
    ADBServerDevice, Result, RustADBError,
};

#[derive(Debug)]
pub struct AdbStatResponse {
    pub file_perm: u32,
    pub file_size: u32,
    pub mod_time: u32,
}

impl From<[u8; 12]> for AdbStatResponse {
    fn from(value: [u8; 12]) -> Self {
        Self {
            file_perm: LittleEndian::read_u32(&value[0..4]),
            file_size: LittleEndian::read_u32(&value[4..8]),
            mod_time: LittleEndian::read_u32(&value[8..]),
        }
    }
}

impl Display for AdbStatResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = UNIX_EPOCH + Duration::from_secs(self.mod_time.into());
        // Create DateTime from SystemTime
        let datetime = DateTime::<Utc>::from(d);

        writeln!(f, "File permissions: {}", self.file_perm)?;
        writeln!(f, "File size: {} bytes", self.file_size)?;
        write!(
            f,
            "Modification time: {}",
            datetime.format("%Y-%m-%d %H:%M:%S.%f %Z")
        )?;
        Ok(())
    }
}

impl ADBServerDevice {
    fn handle_stat_command<S: AsRef<str>>(&mut self, path: S) -> Result<AdbStatResponse> {
        let mut len_buf = [0_u8; 4];
        LittleEndian::write_u32(&mut len_buf, path.as_ref().len() as u32);

        // 4 bytes of command name is already sent by send_sync_request
        self.get_transport_mut()
            .get_raw_connection()?
            .write_all(&len_buf)?;
        self.get_transport_mut()
            .get_raw_connection()?
            .write_all(path.as_ref().to_string().as_bytes())?;

        // Reads returned status code from ADB server
        let mut response = [0_u8; 4];
        self.get_transport_mut()
            .get_raw_connection()?
            .read_exact(&mut response)?;
        match std::str::from_utf8(response.as_ref())? {
            "STAT" => {
                let mut data = [0_u8; 12];
                self.get_transport_mut()
                    .get_raw_connection()?
                    .read_exact(&mut data)?;

                Ok(data.into())
            }
            x => Err(RustADBError::UnknownResponseType(format!(
                "Unknown response {}",
                x
            ))),
        }
    }

    /// Stat file given as [path] on the device.
    pub fn stat<A: AsRef<str>>(&mut self, path: A) -> Result<AdbStatResponse> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;

        // Set device in SYNC mode
        self.get_transport_mut()
            .send_adb_request(AdbServerCommand::Sync)?;

        // Send a "Stat" command
        self.get_transport_mut()
            .send_sync_request(SyncCommand::Stat)?;

        self.handle_stat_command(path)
    }
}
