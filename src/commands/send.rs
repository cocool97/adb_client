use crate::{
    models::{AdbCommand, AdbRequestStatus, SyncCommand},
    AdbTcpConnexion, Result, RustADBError,
};
use byteorder::{ByteOrder, LittleEndian};
use std::{
    convert::TryInto,
    io::{Read, Write},
    str::{self, FromStr},
    time::SystemTime,
};

impl AdbTcpConnexion {
    /// Sends [stream] to [path] on the device.
    pub fn send<S: ToString, A: AsRef<str>>(
        &mut self,
        serial: Option<S>,
        stream: &mut dyn Read,
        path: A,
    ) -> Result<()> {
        self.new_connection()?;

        match serial {
            None => Self::send_adb_request(&mut self.tcp_stream, AdbCommand::TransportAny)?,
            Some(serial) => Self::send_adb_request(
                &mut self.tcp_stream,
                AdbCommand::TransportSerial(serial.to_string()),
            )?,
        }

        // Set device in SYNC mode
        Self::send_adb_request(&mut self.tcp_stream, AdbCommand::Sync)?;

        // Send a send command
        self.send_sync_request(SyncCommand::Send(stream, path.as_ref()))?;

        self.handle_send_command(stream, path)
    }

    fn handle_send_command<S: AsRef<str>>(&mut self, input: &mut dyn Read, to: S) -> Result<()> {
        // Append the permission flags to the filename
        let to = to.as_ref().to_string() + ",0777";

        // The name of command is already sent by send_sync_request
        let mut len_buf = [0_u8; 4];
        LittleEndian::write_u32(&mut len_buf, to.len() as u32);
        self.tcp_stream.write_all(&len_buf)?;

        // Send appends the filemode to the string sent
        self.tcp_stream.write_all(to.as_bytes())?;

        // Then we send the byte data in chunks of up to 64k
        // Chunk looks like 'DATA' <length> <data>
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let bytes_read = input.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            let mut chunk_len_buf = [0_u8; 4];
            LittleEndian::write_u32(&mut chunk_len_buf, bytes_read as u32);
            self.tcp_stream.write_all(b"DATA")?;
            self.tcp_stream.write_all(&chunk_len_buf)?;
            self.tcp_stream.write_all(&buffer[..bytes_read])?;
        }

        // When we are done sending, we send 'DONE' <last modified time>
        // Re-use len_buf to send the last modified time
        let last_modified = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n,
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };
        LittleEndian::write_u32(&mut len_buf, last_modified.as_secs() as u32);
        self.tcp_stream.write_all(b"DONE")?;
        self.tcp_stream.write_all(&len_buf)?;

        // We expect 'OKAY' response from this
        let mut request_status = [0; 4];
        self.tcp_stream.read_exact(&mut request_status)?;

        match AdbRequestStatus::from_str(str::from_utf8(request_status.as_ref())?)? {
            AdbRequestStatus::Fail => {
                // We can keep reading to get further details
                let length = Self::get_body_length(&mut self.tcp_stream)?;

                let mut body = vec![
                    0;
                    length
                        .try_into()
                        .map_err(|_| RustADBError::ConvertionError)?
                ];
                if length > 0 {
                    self.tcp_stream.read_exact(&mut body)?;
                }

                Err(RustADBError::ADBRequestFailed(String::from_utf8(body)?))
            }
            AdbRequestStatus::Okay => Ok(()),
        }
    }
}
