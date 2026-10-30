use std::{
    io::{BufReader, BufWriter, Read, Write},
    time::SystemTime,
};

use byteorder::{LittleEndian, ReadBytesExt};

use super::{ADBSyncSession, MAX_BUFFER_SIZE, read_command_id, response_error, send_sync_request};
use crate::{Result, RustADBError, models::SyncCommand};

impl ADBSyncSession {
    /// Sends the content of `input` to `path` on the device.
    pub fn push(&mut self, input: &mut dyn Read, path: &str) -> Result<()> {
        log::debug!("Sending data to {path}");

        self.run(|mut stream| {
            // Append the permission flags to the filename
            send_sync_request(
                stream,
                &SyncCommand::Send,
                format!("{path},0777").as_bytes(),
            )?;

            let mut writer =
                BufWriter::with_capacity(MAX_BUFFER_SIZE, ADBSendCommandWriter::new(stream));
            std::io::copy(
                &mut BufReader::with_capacity(MAX_BUFFER_SIZE, input),
                &mut writer,
            )?;
            writer.flush()?;

            // Copy is finished, we can now notify as finished
            // Have to send DONE + file mtime (as a u32)
            let Ok(last_modified) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) else {
                return Err(RustADBError::ADBRequestFailed(
                    "SystemTime before UNIX EPOCH!".into(),
                ));
            };
            let mut done_buffer = Vec::with_capacity(8);
            done_buffer.extend_from_slice(b"DONE");
            done_buffer.extend_from_slice(&u32::try_from(last_modified.as_secs())?.to_le_bytes());
            stream.write_all(&done_buffer)?;

            match read_command_id(stream)? {
                id if &id == b"OKAY" => {
                    // Followed by the (empty) message length
                    stream.read_u32::<LittleEndian>()?;
                    Ok(())
                }
                id => Err(response_error(stream, id)),
            }
        })
    }
}

/// Internal structure wrapping a [`std::io::Write`] and hiding underlying protocol logic.
struct ADBSendCommandWriter<W: Write> {
    inner: W,
}

impl<W: Write> ADBSendCommandWriter<W> {
    pub const fn new(inner: W) -> Self {
        Self { inner }
    }
}

impl<W: Write> Write for ADBSendCommandWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // A chunk cannot exceed maximum `DATA` size, remaining bytes will be sent by next call
        let buf = &buf[..buf.len().min(MAX_BUFFER_SIZE)];
        let chunk_len = u32::try_from(buf.len()).map_err(std::io::Error::other)?;

        // 8 = "DATA".len() + sizeof(u32)
        let mut buffer = Vec::with_capacity(8 + buf.len());
        buffer.extend_from_slice(b"DATA");
        buffer.extend_from_slice(&chunk_len.to_le_bytes());
        buffer.extend_from_slice(buf);

        self.inner.write_all(&buffer)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
