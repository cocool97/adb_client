use crate::{
    models::{AdbCommand, SyncCommand},
    AdbTcpConnexion, Result, RustADBError,
};
use byteorder::{ByteOrder, LittleEndian};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

impl AdbTcpConnexion {
    /// Pulls [path] to [filename] from the device.
    pub fn pull<S: ToString, A: AsRef<str>>(
        &mut self,
        serial: Option<S>,
        path: A,
        filename: A,
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

        // Send a recv command
        self.send_sync_request(SyncCommand::Recv(
            path.as_ref(),
            filename.as_ref().to_string(), // Fix this uglyness by using IO Traits
        ))?;

        self.handle_recv_command(path, filename)
    }

    fn handle_recv_command<S: AsRef<str>>(&mut self, from: S, to: S) -> Result<()> {
        // First send 8 byte common header
        let mut len_buf = [0_u8; 4];
        LittleEndian::write_u32(&mut len_buf, from.as_ref().len() as u32);
        self.tcp_stream.write_all(&len_buf)?;
        self.tcp_stream.write_all(from.as_ref().as_bytes())?;

        // Then we receive the byte data in chunks of up to 64k
        // Chunk looks like 'DATA' <length> <data>
        let mut output = File::create(Path::new(to.as_ref())).unwrap();
        // Should this be Boxed?
        let mut buffer = [0_u8; 64 * 1024];
        let mut data_header = [0_u8; 4]; // DATA
        let mut len_header = [0_u8; 4]; // <len>
        loop {
            self.tcp_stream.read_exact(&mut data_header)?;
            // Check if data_header is DATA or DONE
            if data_header.eq(b"DATA") {
                self.tcp_stream.read_exact(&mut len_header)?;
                let length: usize = LittleEndian::read_u32(&len_header).try_into().unwrap();
                self.tcp_stream.read_exact(&mut buffer[..length])?;
                output.write_all(&buffer)?;
            } else if data_header.eq(b"DONE") {
                // We're done here
                break;
            } else if data_header.eq(b"FAIL") {
                // Handle fail
                self.tcp_stream.read_exact(&mut len_header)?;
                let length: usize = LittleEndian::read_u32(&len_header).try_into().unwrap();
                self.tcp_stream.read_exact(&mut buffer[..length])?;
                Err(RustADBError::ADBRequestFailed(String::from_utf8(
                    buffer[..length].to_vec(),
                )?))?;
            } else {
                panic!("Unknown response from device {:#?}", data_header);
            }
        }

        // Connection should've left SYNC by now
        Ok(())
    }
}
