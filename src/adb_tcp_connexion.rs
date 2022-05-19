use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    str,
    str::FromStr,
};

use crate::{
    models::{AdbCommand, AdbRequestStatus},
    Result, RustADBError,
};

/// Represents an ADB-over-TCP connexion.
#[derive(Debug)]
pub struct AdbTcpConnexion {
    pub(crate) socket_addr: SocketAddrV4,
    pub(crate) port: u16,
    pub(crate) address: Ipv4Addr,
}

impl AdbTcpConnexion {
    /// Instantiates a new instance of [AdbTcpConnexion]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a custom listening port for ADB server.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self.socket_addr = SocketAddrV4::new(self.address, self.port);
        self
    }

    /// Sets a custom listening address for ADB server.
    pub fn address(mut self, address: Ipv4Addr) -> Self {
        self.address = address;
        self.socket_addr = SocketAddrV4::new(self.address, self.port);
        self
    }

    pub(crate) fn proxy_connexion(
        &self,
        adb_command: AdbCommand,
        with_response: bool,
    ) -> Result<Vec<u8>> {
        let mut tcp_stream = TcpStream::connect(self.socket_addr)?;

        Self::send_adb_request(&mut tcp_stream, adb_command)?;

        if with_response {
            let length = Self::get_body_length(&mut tcp_stream)?;
            let mut body = vec![
                0;
                length
                    .try_into()
                    .map_err(|_| RustADBError::ConvertionError)?
            ];
            if length > 0 {
                tcp_stream.read_exact(&mut body)?;
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

    pub(crate) fn get_body_length(tcp_stream: &mut TcpStream) -> Result<u32> {
        let mut length = [0; 4];
        tcp_stream.read_exact(&mut length)?;

        Ok(u32::from_str_radix(str::from_utf8(&length)?, 16)?)
    }
}

impl Default for AdbTcpConnexion {
    fn default() -> Self {
        let default_port: u16 = 5037;
        let default_address = Ipv4Addr::new(127, 0, 0, 1);

        Self {
            socket_addr: SocketAddrV4::new(default_address, default_port),
            address: default_address,
            port: default_port,
        }
    }
}
