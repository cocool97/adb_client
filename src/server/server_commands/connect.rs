use crate::{models::AdbCommand,  ADBServer, Result, RustADBError};
use std::net::Ipv4Addr;

impl ADBServer {
    /// Connect device over tcp with address and port
    pub fn connect_device(&mut self, address: Ipv4Addr, port: u16) -> Result<()> {
        let response = self.connect()?.proxy_connection(AdbCommand::Connect(address, port), true)?;

        match String::from_utf8(response).unwrap() {
            s if s.starts_with("connected to") => Ok(()),
            s if s.starts_with("failed to connect to") => Err(RustADBError::ADBDeviceNotPaired),
            s => Err(RustADBError::ADBRequestFailed(s))
        }
    }
}
