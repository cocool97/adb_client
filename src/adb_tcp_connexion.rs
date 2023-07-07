use byteorder::{ByteOrder, LittleEndian};
use std::{
    fs::File,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    path::{Path, PathBuf},
    str,
    str::FromStr,
    time::SystemTime,
};

use crate::{
    models::{AdbCommand, AdbRequestStatus, SyncCommand},
    Result, RustADBError,
};

/// Represents an ADB-over-TCP connexion.
#[derive(Debug)]
pub struct AdbTcpConnexion {
    pub(crate) socket_addr: SocketAddrV4,
    pub(crate) tcp_stream: TcpStream,
}

impl AdbTcpConnexion {
    /// Instantiates a new instance of [AdbTcpConnexion]
    pub fn new(address: Ipv4Addr, port: u16) -> Result<Self> {
        let addr = SocketAddrV4::new(address, port);
        Ok(Self {
            socket_addr: addr,
            tcp_stream: TcpStream::connect(addr)?,
        })
    }

    /// Creates a new connection to ADB server.
    ///
    /// Can be used after requests that closes connection.
    pub(crate) fn new_connection(&mut self) -> Result<()> {
        self.tcp_stream = TcpStream::connect(self.socket_addr)?;

        Ok(())
    }

    pub(crate) fn proxy_connexion(
        &mut self,
        adb_command: AdbCommand,
        with_response: bool,
    ) -> Result<Vec<u8>> {
        Self::send_adb_request(&mut self.tcp_stream, adb_command)?;

        if with_response {
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

            Ok(body)
        } else {
            Ok(vec![])
        }
    }

    /// Sends the given [AdbCommand] to ADB server, and checks that the request has been taken in consideration.
    /// If an error occured, a [RustADBError] is returned with the response error string.
    pub(crate) fn send_adb_request(tcp_stream: &mut TcpStream, command: AdbCommand) -> Result<()> {
        let adb_command_string = command.to_string();
        let adb_request = format!("{:04x}{}", adb_command_string.len(), adb_command_string);

        tcp_stream.write_all(adb_request.as_bytes())?;

        // Reads returned status code from ADB server
        let mut request_status = [0; 4];
        tcp_stream.read_exact(&mut request_status)?;

        match AdbRequestStatus::from_str(str::from_utf8(request_status.as_ref())?)? {
            AdbRequestStatus::Fail => {
                // We can keep reading to get further details
                let length = Self::get_body_length(tcp_stream)?;

                let mut body = vec![
                    0;
                    length
                        .try_into()
                        .map_err(|_| RustADBError::ConvertionError)?
                ];
                if length > 0 {
                    tcp_stream.read_exact(&mut body)?;
                }

                Err(RustADBError::ADBRequestFailed(String::from_utf8(body)?))
            }
            AdbRequestStatus::Okay => Ok(()),
        }
    }

    /// Sends the given [SyncCommand] to ADB server, and checks that the request has been taken in consideration.
    /// Note: This function does not take a tcp_stream anymore, as it is already stored in the struct.
    pub(crate) fn send_sync_request(&mut self, command: SyncCommand) -> Result<()> {
        // First 4 bytes are the name of the command we want to send
        // (e.g. "SEND", "RECV", "STAT", "LIST")
        Ok(self.tcp_stream.write_all(command.to_string().as_bytes())?)
    }

    // This command does not seem to work correctly. The devices I test it on just resturn
    // 'DONE' directly without listing anything.
    fn handle_list_command<S: AsRef<str>>(&mut self, path: S) -> Result<()> {
        let mut len_buf = [0_u8; 4];
        LittleEndian::write_u32(&mut len_buf, path.as_ref().len() as u32);

        // 4 bytes of command name is already sent by send_sync_request
        self.tcp_stream.write_all(&len_buf)?;

        // List sends the string of the directory to list, and then the server sends a list of files
        self.tcp_stream
            .write_all(path.as_ref().to_string().as_bytes())?;

        // Reads returned status code from ADB server
        let mut response = [0_u8; 4];
        loop {
            self.tcp_stream.read_exact(&mut response)?;
            match str::from_utf8(response.as_ref())? {
                "DENT" => {
                    // TODO: Move this to a struct that extract this data, but as the device
                    // I test this on does not return anything, I can't test it.
                    let mut file_mod = [0_u8; 4];
                    let mut file_size = [0_u8; 4];
                    let mut mod_time = [0_u8; 4];
                    let mut name_len = [0_u8; 4];
                    self.tcp_stream.read_exact(&mut file_mod)?;
                    self.tcp_stream.read_exact(&mut file_size)?;
                    self.tcp_stream.read_exact(&mut mod_time)?;
                    self.tcp_stream.read_exact(&mut name_len)?;
                    let name_len = LittleEndian::read_u32(&name_len);
                    let mut name_buf = vec![0_u8; name_len as usize];
                    self.tcp_stream.read_exact(&mut name_buf)?;
                }
                "DONE" => {
                    return Ok(());
                }
                x => println!("Unknown response {}", x),
            }
        }
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

    pub(crate) fn get_body_length(tcp_stream: &mut TcpStream) -> Result<u32> {
        let mut length = [0; 4];
        tcp_stream.read_exact(&mut length)?;

        Ok(u32::from_str_radix(str::from_utf8(&length)?, 16)?)
    }
}
