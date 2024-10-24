use crate::models::AdbServerCommand;
use crate::{ADBServer, Result, RustADBError};
use std::net::SocketAddrV4;

impl ADBServer {
    /// Pair device on a specific port with a generated 'code'
    pub fn pair(&mut self, address: SocketAddrV4, code: String) -> Result<()> {
        let response = self
            .connect()?
            .proxy_connection(AdbServerCommand::Pair(address, code), true)?;

        match String::from_utf8(response) {
            Ok(s) if s.starts_with("Successfully paired to") => Ok(()),
            Ok(s) => Err(RustADBError::ADBRequestFailed(s)),
            Err(e) => Err(e.into()),
        }
    }
}
