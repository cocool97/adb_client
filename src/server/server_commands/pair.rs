use crate::models::AdbCommand;
use crate::{ADBServer, Result, RustADBError};
use std::net::SocketAddrV4;

impl ADBServer {
    /// Pair device on a specific port with a generated 'code'
    pub fn pair(&mut self, address: SocketAddrV4, code: u32) -> Result<()> {
        let response = self
            .connect()?
            .proxy_connection(AdbCommand::Pair(address, code), true)?;

        match String::from_utf8(response).unwrap() {
            s if s.starts_with("Successfully paired to") => Ok(()),
            s => Err(RustADBError::ADBRequestFailed(s)),
        }
    }
}
