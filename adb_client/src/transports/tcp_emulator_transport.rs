use std::{
    fs::File,
    io::{BufRead, BufReader, Error, ErrorKind, Read, Write},
    net::{SocketAddrV4, TcpStream},
};

use homedir::my_home;

use super::ADBTransport;
use crate::{Result, RustADBError, emulator_device::ADBEmulatorCommand};

/// Emulator transport running on top on TCP.
#[derive(Debug)]
pub struct TCPEmulatorTransport {
    socket_addr: SocketAddrV4,
    tcp_stream: Option<TcpStream>,
}

impl TCPEmulatorTransport {
    /// Instantiates a new instance of [TCPEmulatorTransport]
    pub fn new(socket_addr: SocketAddrV4) -> Self {
        Self {
            socket_addr,
            tcp_stream: None,
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

    /// Return authentication token stored in $HOME/.emulator_console_auth_token
    pub fn get_authentication_token(&mut self) -> Result<String> {
        let home = match my_home()? {
            Some(home) => home,
            None => return Err(RustADBError::NoHomeDirectory),
        };

        let mut f = File::open(home.join(".emulator_console_auth_token"))?;
        let mut token = String::new();
        f.read_to_string(&mut token)?;

        Ok(token)
    }

    /// Send an authenticate request to this emulator
    pub fn authenticate(&mut self) -> Result<()> {
        let token = self.get_authentication_token()?;
        self.send_command(ADBEmulatorCommand::Authenticate(token))
    }

    /// Send an [ADBEmulatorCommand] to this emulator
    pub(crate) fn send_command(&mut self, command: ADBEmulatorCommand) -> Result<()> {
        let mut connection = self.get_raw_connection()?;

        // Send command
        connection.write_all(command.to_string().as_bytes())?;

        // Check is an error occurred skipping lines depending on command
        self.check_error(command.skip_response_lines())?;

        Ok(())
    }

    fn check_error(&mut self, skipping: u8) -> Result<()> {
        let mut reader = BufReader::new(self.get_raw_connection()?);
        for _ in 0..skipping {
            let mut line = String::new();
            reader.read_line(&mut line)?;
            if line.starts_with("KO:") {
                return Err(RustADBError::ADBRequestFailed(line));
            }
        }

        let mut line = String::new();
        reader.read_line(&mut line)?;

        match line.starts_with("OK") {
            true => Ok(()),
            false => Err(RustADBError::ADBRequestFailed(line)),
        }
    }
}

impl ADBTransport for TCPEmulatorTransport {
    fn disconnect(&mut self) -> Result<()> {
        if let Some(conn) = &mut self.tcp_stream {
            conn.shutdown(std::net::Shutdown::Both)?;
            log::trace!("Disconnected from {}", conn.peer_addr()?);
        }

        Ok(())
    }

    /// Connect to current emulator and authenticate
    fn connect(&mut self) -> Result<()> {
        if self.tcp_stream.is_none() {
            let stream = TcpStream::connect(self.socket_addr)?;

            log::trace!("Successfully connected to {}", self.socket_addr);

            self.tcp_stream = Some(stream.try_clone()?);

            let mut reader = BufReader::new(stream);

            // Android Console: Authentication required
            // Android Console: type 'auth <auth_token>' to authenticate
            // Android Console: you can find your <auth_token> in
            // '/home/xxx/.emulator_console_auth_token'
            for _ in 0..=4 {
                let mut line = String::new();
                reader.read_line(&mut line)?;
            }

            self.authenticate()?;

            log::trace!("Authentication successful");
        }

        Ok(())
    }
}
