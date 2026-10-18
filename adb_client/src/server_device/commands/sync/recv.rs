use std::io::{BufReader, BufWriter, Read, Write};

use byteorder::{LittleEndian, ReadBytesExt};

use super::{ADBSyncSession, MAX_BUFFER_SIZE, send_sync_request};
use crate::{Result, models::SyncCommand};

impl ADBSyncSession<'_> {
    /// Receives the file at `path` on the device, and writes its content into `output`.
    pub fn pull(&mut self, path: &str, output: &mut dyn Write) -> Result<()> {
        self.run(|stream| {
            send_sync_request(stream, &SyncCommand::Recv, path.as_bytes())?;

            let mut writer = BufWriter::with_capacity(MAX_BUFFER_SIZE, output);
            std::io::copy(
                &mut BufReader::with_capacity(MAX_BUFFER_SIZE, ADBRecvCommandReader::new(stream)),
                &mut writer,
            )?;
            writer.flush()?;

            Ok(())
        })
    }
}

/// Internal structure wrapping a [`std::io::Read`] and hiding underlying protocol logic.
struct ADBRecvCommandReader<R: Read> {
    inner: R,
    remaining_data_bytes_to_read: usize,
    done: bool,
}

impl<R: Read> ADBRecvCommandReader<R> {
    pub const fn new(inner: R) -> Self {
        Self {
            inner,
            remaining_data_bytes_to_read: 0,
            done: false,
        }
    }
}

impl<R: Read> Read for ADBRecvCommandReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // In case of a "DATA" header, we may not have enough space in `buf` to fill it with "length" bytes coming from device.
        // `remaining_data_bytes_to_read` represents how many bytes are still left to read before receiving another header.
        while self.remaining_data_bytes_to_read == 0 {
            if self.done {
                return Ok(0);
            }

            let mut header = [0_u8; 4];
            self.inner.read_exact(&mut header)?;

            match &header {
                b"DATA" => {
                    self.remaining_data_bytes_to_read =
                        self.inner.read_u32::<LittleEndian>()? as usize;
                }
                b"DONE" => {
                    // Followed by an unused length, that must be consumed to keep connection usable
                    self.inner.read_u32::<LittleEndian>()?;
                    self.done = true;
                }
                b"FAIL" => {
                    let length = self.inner.read_u32::<LittleEndian>()? as usize;
                    let mut error_msg = vec![0; length];
                    self.inner.read_exact(&mut error_msg)?;

                    return Err(std::io::Error::other(format!(
                        "ADB request failed: {}",
                        String::from_utf8_lossy(&error_msg)
                    )));
                }
                _ => {
                    return Err(std::io::Error::other(format!(
                        "Unknown response from device {header:#?}"
                    )));
                }
            }
        }

        // Computing minimum to ensure to stop reading before next header...
        let data_to_read = std::cmp::min(self.remaining_data_bytes_to_read, buf.len());
        let effective_read = self.inner.read(&mut buf[..data_to_read])?;
        if effective_read == 0 {
            // Returning 0 would be taken as a regular end of file, leaving a truncated output
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        self.remaining_data_bytes_to_read -= effective_read;

        Ok(effective_read)
    }
}
