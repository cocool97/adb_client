use crate::{models::AdbCommand,  AdbTcpConnection, Result, RustADBError};
use std::net::Ipv4Addr;

impl AdbTcpConnection {
    /// Connect device over tcp with address and port
    pub fn connect(&mut self, address: Ipv4Addr, port: u16) -> Result<()> {
        let response = self.proxy_connection(AdbCommand::Connect(address, port), true)?;

        match String::from_utf8(response).unwrap() {
            s if s.starts_with("connected to") => Ok(()),
            s => Err(RustADBError::ADBRequestFailed(s))
        }
    }
}
