use std::io::{Error, ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::str::FromStr;

use byteorder::{ByteOrder, LittleEndian};

use crate::models::{AdbRequestStatus, SyncCommand};
use crate::{ADBTransport, models::AdbServerCommand};
use crate::{Result, RustADBError};

const DEFAULT_SERVER_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const DEFAULT_SERVER_PORT: u16 = 5037;

/// Server transport running on top on TCP
#[derive(Debug)]
pub struct TCPServerTransport {
    socket_addr: SocketAddrV4,
    tcp_stream: Option<TcpStream>,
}

impl Default for TCPServerTransport {
    fn default() -> Self {
        Self::new(SocketAddrV4::new(DEFAULT_SERVER_IP, DEFAULT_SERVER_PORT))
    }
}

impl TCPServerTransport {
    /// Instantiates a new instance of [TCPServerTransport]
    pub fn new(socket_addr: SocketAddrV4) -> Self {
        Self {
            socket_addr,
            tcp_stream: None,
        }
    }

    /// Instantiate a new instance of [TCPServerTransport] using given address, or default if not specified.
    pub fn new_or_default(socket_addr: Option<SocketAddrV4>) -> Self {
        match socket_addr {
            Some(s) => Self::new(s),
            None => Self::default(),
        }
    }

    /// Get underlying [SocketAddrV4]
    pub fn get_socketaddr(&self) -> SocketAddrV4 {
        self.socket_addr
    }

    pub(crate) fn proxy_connection(
        &mut self,
        adb_command: AdbServerCommand,
        with_response: bool,
    ) -> Result<Vec<u8>> {
        self.send_adb_request(adb_command)?;

        if with_response {
            let length = self.get_hex_body_length()?;
            let mut body = vec![
                0;
                length
                    .try_into()
                    .map_err(|_| RustADBError::ConversionError)?
            ];
            if length > 0 {
                self.get_raw_connection()?.read_exact(&mut body)?;
            }

            Ok(body)
        } else {
            Ok(vec![])
        }
    }

    pub(crate) fn get_raw_connection(&self) -> Result<&TcpStream> {
        self.tcp_stream
            .as_ref()
            .ok_or(RustADBError::IOError(Error::new(
                ErrorKind::NotConnected,
                "not connected",
            )))
    }

    /// Gets the body length from hexadecimal value
    pub(crate) fn get_hex_body_length(&mut self) -> Result<u32> {
        let length_buffer = self.read_body_length()?;
        Ok(u32::from_str_radix(
            std::str::from_utf8(&length_buffer)?,
            16,
        )?)
    }

    /// Send the given [SyncCommand] to ADB server, and checks that the request has been taken in consideration.
    pub(crate) fn send_sync_request(&mut self, command: SyncCommand) -> Result<()> {
        // First 4 bytes are the name of the command we want to send
        // (e.g. "SEND", "RECV", "STAT", "LIST")
        Ok(self
            .get_raw_connection()?
            .write_all(command.to_string().as_bytes())?)
    }

    /// Gets the body length from a LittleEndian value
    pub(crate) fn get_body_length(&mut self) -> Result<u32> {
        let length_buffer = self.read_body_length()?;
        Ok(LittleEndian::read_u32(&length_buffer))
    }

    /// Read 4 bytes representing body length
    fn read_body_length(&mut self) -> Result<[u8; 4]> {
        let mut length_buffer = [0; 4];
        self.get_raw_connection()?.read_exact(&mut length_buffer)?;

        Ok(length_buffer)
    }

    /// Send the given [AdbCommand] to ADB server, and checks that the request has been taken in consideration.
    /// If an error occurred, a [RustADBError] is returned with the response error string.
    pub(crate) fn send_adb_request(&mut self, command: AdbServerCommand) -> Result<()> {
        let adb_command_string = command.to_string();
        let adb_request = format!("{:04x}{}", adb_command_string.len(), adb_command_string);

        self.get_raw_connection()?
            .write_all(adb_request.as_bytes())?;

        self.read_adb_response()
    }

    /// Read a response from ADB server
    pub(crate) fn read_adb_response(&mut self) -> Result<()> {
        // Reads returned status code from ADB server
        let mut request_status = [0; 4];
        self.get_raw_connection()?.read_exact(&mut request_status)?;

        match AdbRequestStatus::from_str(std::str::from_utf8(request_status.as_ref())?)? {
            AdbRequestStatus::Fail => {
                // We can keep reading to get further details
                let length = self.get_hex_body_length()?;

                let mut body = vec![
                    0;
                    length
                        .try_into()
                        .map_err(|_| RustADBError::ConversionError)?
                ];
                if length > 0 {
                    self.get_raw_connection()?.read_exact(&mut body)?;
                }

                Err(RustADBError::ADBRequestFailed(String::from_utf8(body)?))
            }
            AdbRequestStatus::Okay => Ok(()),
        }
    }
}

impl ADBTransport for TCPServerTransport {
    fn disconnect(&mut self) -> Result<()> {
        if let Some(conn) = &mut self.tcp_stream {
            conn.shutdown(std::net::Shutdown::Both)?;
            log::trace!("Disconnected from {}", conn.peer_addr()?);
        }

        Ok(())
    }

    fn connect(&mut self) -> Result<()> {
        if let Some(previous) = &self.tcp_stream {
            // Ignoring underlying error, we will recreate a new connection
            let _ = previous.shutdown(std::net::Shutdown::Both);
        }
        let tcp_stream = TcpStream::connect(self.socket_addr)?;
        tcp_stream.set_nodelay(true)?;
        self.tcp_stream = Some(tcp_stream);
        log::trace!("Successfully connected to {}", self.socket_addr);

        Ok(())
    }
}
