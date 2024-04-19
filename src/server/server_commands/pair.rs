use std::net::Ipv4Addr;
use crate::{ADBServer, Result, RustADBError};
use crate::models::AdbCommand;

impl ADBServer {
    /// Pair device on a specific port with a generated 'code'
    pub fn pair(&mut self, address: Ipv4Addr, port: u16, code: u32) -> Result<()> {
        let response = self.connect()?.proxy_connection(AdbCommand::Pair(address, port, code), true)?;

        match String::from_utf8(response).unwrap() {
            s if s.starts_with("Successfully paired to") => Ok(()),
            s => Err(RustADBError::ADBRequestFailed(s))
        }
    }
}
