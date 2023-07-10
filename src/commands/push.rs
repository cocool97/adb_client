use crate::{
    models::{AdbCommand, AdbRequestStatus, SyncCommand},
    AdbTcpConnexion, Result, RustADBError,
};
use byteorder::{ByteOrder, LittleEndian};
use std::{
    convert::TryInto,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    str::{self, FromStr},
    time::SystemTime,
};

impl AdbTcpConnexion {
    /// Pushes [filename] to [path] on the device.
    pub fn push<S: ToString, A: AsRef<str>>(
        &mut self,
        serial: Option<S>,
        filename: A,
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
        self.send_sync_request(SyncCommand::Send(
            filename.as_ref(),
            path.as_ref().to_string(), // Fix this uglyness by using IO Traits
        ))?;

        self.handle_send_command(filename, path)
    }

    fn handle_send_command<S: AsRef<str>>(&mut self, from: S, to: S) -> Result<()> {
        // Append the filename from 'from' to the path of 'to'
        // FIXME: This should only be done if 'to' doesn't already contain a filename
        // I guess we need to STAT the to file first to check this
        // but we can't just call this, as the device needs to be put into SYNC mode first
        // and that is done separately from this function
        // If we'd make the input here to be a IO trait, then we wouldn't need to care
        // about the name as the 'to' would need to contain the full name as the caller
        // would be the one handling the naming
        let mut to = PathBuf::from(to.as_ref());
        to.push(
            Path::new(from.as_ref())
                .file_name()
                .ok_or(RustADBError::ADBRequestFailed(
                    "Could not get filename...".to_string(),
                ))?,
        );
        let to = to.display().to_string() + ",0777";

        // The name of command is already sent by send_sync_request
        let mut len_buf = [0_u8; 4];
        LittleEndian::write_u32(&mut len_buf, to.len() as u32);
        self.tcp_stream.write_all(&len_buf)?;

        // Send appends the filemode to the string sent
        self.tcp_stream.write_all(to.as_bytes())?;

        // Then we send the byte data in chunks of up to 64k
        // Chunk looks like 'DATA' <length> <data>
        let mut file = File::open(Path::new(from.as_ref()))?;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let bytes_read = file.read(&mut buffer)?;
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
        let metadata = std::fs::metadata(Path::new(from.as_ref()))?;
        let last_modified = match metadata.modified()?.duration_since(SystemTime::UNIX_EPOCH) {
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
