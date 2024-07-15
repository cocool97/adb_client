use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    str,
    str::FromStr,
};

use byteorder::{ByteOrder, LittleEndian};

use crate::{
    models::{AdbCommand, AdbRequestStatus, SyncCommand},
    Result, RustADBError,
};

/// Represents an ADB-over-TCP connection.
#[derive(Debug)]
pub struct AdbTcpConnection {
    pub(crate) socket_addr: SocketAddrV4,
    pub(crate) tcp_stream: TcpStream,
}

impl AdbTcpConnection {
    /// Instantiates a new instance of [AdbTcpConnection]
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

    pub(crate) fn proxy_connection(
        &mut self,
        adb_command: AdbCommand,
        with_response: bool,
        fresh_connection: bool
    ) -> Result<Vec<u8>> {
        self.send_adb_request(adb_command, fresh_connection)?;

        if with_response {
            let length = self.get_hex_body_length()?;
            let mut body = vec![
                0;
                length
                    .try_into()
                    .map_err(|_| RustADBError::ConversionError)?
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
    /// If an error occurred, a [RustADBError] is returned with the response error string.
    pub(crate) fn send_adb_request(
        &mut self,
        command: AdbCommand,
        fresh_connection: bool,
    ) -> Result<()> {
        if fresh_connection {
            // Recreate a new connection (likely because command does not need "state" server side)
            self.new_connection()?;
        }

        let adb_command_string = command.to_string();
        let adb_request = format!("{:04x}{}", adb_command_string.len(), adb_command_string);

        self.tcp_stream.write_all(adb_request.as_bytes())?;

        // Reads returned status code from ADB server
        let mut request_status = [0; 4];
        self.tcp_stream.read_exact(&mut request_status)?;

        match AdbRequestStatus::from_str(str::from_utf8(request_status.as_ref())?)? {
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
                    self.tcp_stream.read_exact(&mut body)?;
                }

                Err(RustADBError::ADBRequestFailed(String::from_utf8(body)?))
            }
            AdbRequestStatus::Okay => Ok(()),
        }
    }

    /// Sends the given [SyncCommand] to ADB server, and checks that the request has been taken in consideration.
    pub(crate) fn send_sync_request(&mut self, command: SyncCommand) -> Result<()> {
        // First 4 bytes are the name of the command we want to send
        // (e.g. "SEND", "RECV", "STAT", "LIST")
        Ok(self.tcp_stream.write_all(command.to_string().as_bytes())?)
    }

    /// Gets the body length from hexadecimal value
    pub(crate) fn get_hex_body_length(&mut self) -> Result<u32> {
        let length_buffer = self.read_body_length()?;
        Ok(u32::from_str_radix(str::from_utf8(&length_buffer)?, 16)?)
    }

    /// Gets the body length from a LittleEndian value
    pub(crate) fn get_body_length(&mut self) -> Result<u32> {
        let length_buffer = self.read_body_length()?;
        Ok(LittleEndian::read_u32(&length_buffer))
    }

    /// Read 4 bytes representing body length
    fn read_body_length(&mut self) -> Result<[u8; 4]> {
        let mut length_buffer = [0; 4];
        self.tcp_stream.read_exact(&mut length_buffer)?;

        Ok(length_buffer)
    }
}
