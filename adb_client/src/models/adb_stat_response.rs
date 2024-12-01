use byteorder::ByteOrder;
use chrono::{DateTime, Utc};
use std::{
    fmt::Display,
    time::{Duration, UNIX_EPOCH},
};

use byteorder::LittleEndian;
use serde::{Deserialize, Serialize};

/// Represents a `stat` response
#[derive(Debug, Deserialize, Serialize)]
pub struct AdbStatResponse {
    /// File permissions
    pub file_perm: u32,
    /// File size, in bytes
    pub file_size: u32,
    /// File modification time
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
