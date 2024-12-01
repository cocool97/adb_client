use crate::{
    constants,
    models::{AdbRequestStatus, AdbServerCommand, SyncCommand},
    ADBServerDevice, Result, RustADBError,
};
use std::{
    convert::TryInto,
    io::{BufReader, BufWriter, Read, Write},
    str::{self, FromStr},
    time::SystemTime,
};

/// Internal structure wrapping a [std::io::Write] and hiding underlying protocol logic.
struct ADBSendCommandWriter<W: Write> {
    inner: W,
}

impl<W: Write> ADBSendCommandWriter<W> {
    pub fn new(inner: W) -> Self {
        ADBSendCommandWriter { inner }
    }
}

impl<W: Write> Write for ADBSendCommandWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let chunk_len = buf.len() as u32;

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

impl ADBServerDevice {
    /// Send stream to path on the device.
    pub fn push<R: Read, A: AsRef<str>>(&mut self, stream: R, path: A) -> Result<()> {
        log::info!("Sending data to {}", path.as_ref());
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;

        // Set device in SYNC mode
        self.get_transport_mut()
            .send_adb_request(AdbServerCommand::Sync)?;

        // Send a send command
        self.get_transport_mut()
            .send_sync_request(SyncCommand::Send)?;

        self.handle_send_command(stream, path)
    }

    fn handle_send_command<R: Read, S: AsRef<str>>(&mut self, input: R, to: S) -> Result<()> {
        // Append the permission flags to the filename
        let to = to.as_ref().to_string() + ",0777";

        let mut raw_connection = self.get_transport_mut().get_raw_connection()?;

        // The name of the command is already sent by get_transport()?.send_sync_request
        let to_as_bytes = to.as_bytes();
        let mut buffer = Vec::with_capacity(4 + to_as_bytes.len());
        buffer.extend_from_slice(&(to.len() as u32).to_le_bytes());
        buffer.extend_from_slice(to_as_bytes);
        raw_connection.write_all(&buffer)?;

        let writer = ADBSendCommandWriter::new(raw_connection);

        std::io::copy(
            &mut BufReader::with_capacity(constants::BUFFER_SIZE, input),
            &mut BufWriter::with_capacity(constants::BUFFER_SIZE, writer),
        )?;

        // Copy is finished, we can now notify as finished
        // Have to send DONE + file mtime
        let last_modified = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n,
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };

        let mut done_buffer = Vec::with_capacity(8);
        done_buffer.extend_from_slice(b"DONE");
        done_buffer.extend_from_slice(&last_modified.as_secs().to_le_bytes());
        raw_connection.write_all(&done_buffer)?;

        // We expect 'OKAY' response from this
        let mut request_status = [0; 4];
        raw_connection.read_exact(&mut request_status)?;

        match AdbRequestStatus::from_str(str::from_utf8(&request_status)?)? {
            AdbRequestStatus::Fail => {
                // We can keep reading to get further details
                let length = self.get_transport_mut().get_body_length()?;

                let mut body = vec![
                    0;
                    length
                        .try_into()
                        .map_err(|_| RustADBError::ConversionError)?
                ];
                if length > 0 {
                    self.get_transport()
                        .get_raw_connection()?
                        .read_exact(&mut body)?;
                }

                Err(RustADBError::ADBRequestFailed(String::from_utf8(body)?))
            }
            AdbRequestStatus::Okay => Ok(()),
        }
    }
}
