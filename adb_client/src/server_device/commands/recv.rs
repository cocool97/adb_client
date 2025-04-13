use crate::{
    ADBServerDevice, Result, constants,
    models::{AdbServerCommand, SyncCommand},
};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{BufReader, BufWriter, Read, Write};

/// Internal structure wrapping a [std::io::Read] and hiding underlying protocol logic.
struct ADBRecvCommandReader<R: Read> {
    inner: R,
    remaining_data_bytes_to_read: usize,
}

impl<R: Read> ADBRecvCommandReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            remaining_data_bytes_to_read: 0,
        }
    }
}

impl<R: Read> Read for ADBRecvCommandReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // In case of a "DATA" header, we may not have enough space in `buf` to fill it with "length" bytes coming from device.
        // `remaining_data_bytes_to_read` represents how many bytes are still left to read before receiving another header.
        if self.remaining_data_bytes_to_read == 0 {
            let mut header = [0_u8; 4];
            self.inner.read_exact(&mut header)?;

            match &header[..] {
                b"DATA" => {
                    let length = self.inner.read_u32::<LittleEndian>()? as usize;
                    let effective_read = self.inner.read(&mut buf[0..length])?;
                    self.remaining_data_bytes_to_read = length - effective_read;

                    Ok(effective_read)
                }
                b"DONE" => Ok(0),
                b"FAIL" => {
                    let length = self.inner.read_u32::<LittleEndian>()? as usize;
                    let mut error_msg = vec![0; length];
                    self.inner.read_exact(&mut error_msg)?;

                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "ADB request failed: {}",
                            String::from_utf8_lossy(&error_msg)
                        ),
                    ))
                }
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Unknown response from device {:#?}", header),
                )),
            }
        } else {
            // Computing minimum to ensure to stop reading before next header...
            let data_to_read = std::cmp::min(self.remaining_data_bytes_to_read, buf.len());
            self.inner.read_exact(&mut buf[..data_to_read])?;

            self.remaining_data_bytes_to_read -= self.remaining_data_bytes_to_read;

            Ok(data_to_read)
        }
    }
}

impl ADBServerDevice {
    /// Receives path to stream from the device.
    pub fn pull(&mut self, path: &dyn AsRef<str>, stream: &mut dyn Write) -> Result<()> {
        self.set_serial_transport()?;

        // Set device in SYNC mode
        self.transport.send_adb_request(AdbServerCommand::Sync)?;

        // Send a recv command
        self.transport.send_sync_request(SyncCommand::Recv)?;

        self.handle_recv_command(path, stream)
    }

    fn handle_recv_command<S: AsRef<str>>(
        &mut self,
        from: S,
        output: &mut dyn Write,
    ) -> Result<()> {
        let mut raw_connection = self.transport.get_raw_connection()?;

        let from_as_bytes = from.as_ref().as_bytes();
        let mut buffer = Vec::with_capacity(4 + from_as_bytes.len());
        buffer.extend_from_slice(&(from.as_ref().len() as u32).to_le_bytes());
        buffer.extend_from_slice(from_as_bytes);
        raw_connection.write_all(&buffer)?;

        let reader = ADBRecvCommandReader::new(raw_connection);
        std::io::copy(
            &mut BufReader::with_capacity(constants::BUFFER_SIZE, reader),
            &mut BufWriter::with_capacity(constants::BUFFER_SIZE, output),
        )?;

        // Connection should've been left in SYNC mode by now
        Ok(())
    }
}
