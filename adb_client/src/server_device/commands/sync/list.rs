use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

use super::{ADBSyncSession, read_command_id, response_error, send_sync_request};
use crate::{
    Result,
    models::{ADBListItem, ADBListItemType, SyncCommand},
};

impl ADBSyncSession {
    /// Lists files in path on the device.
    /// note: path uses internal file paths, so Documents is at /storage/emulated/0/Documents
    pub fn list(&mut self, path: &str) -> Result<Vec<ADBListItemType>> {
        // TODO: use LIS2 to support files over 2.14 GB in size.
        // SEE: https://github.com/cstyan/adbDocumentation?tab=readme-ov-file#adb-list
        self.run(|mut stream| {
            send_sync_request(stream, &SyncCommand::List, path.as_bytes())?;

            let mut list_items = Vec::new();
            loop {
                let id = read_command_id(stream)?;
                if &id != b"DENT" && &id != b"DONE" {
                    return Err(response_error(stream, id));
                }

                // Every entry, including the final `DONE` one, carries mode, size, time and name length
                let mode = stream.read_u32::<LittleEndian>()?;
                let size = stream.read_u32::<LittleEndian>()?;
                let time = stream.read_u32::<LittleEndian>()?;
                let name_len = stream.read_u32::<LittleEndian>()?;

                if &id == b"DONE" {
                    return Ok(list_items);
                }

                let mut name_buf = vec![0_u8; usize::try_from(name_len)?];
                stream.read_exact(&mut name_buf)?;
                let name = String::from_utf8(name_buf)?;

                // First 9 bits are the file permissions
                let permissions = mode & 0b1_1111_1111;

                let entry = ADBListItem {
                    name,
                    time,
                    permissions,
                    size,
                };

                list_items.push(ADBListItemType::from_mode_and_entry(mode, entry));
            }
        })
    }
}
