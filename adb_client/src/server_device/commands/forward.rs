use std::io::Read;

use crate::{
    Result, RustADBError,
    models::{ADBCommand, ADBLocalCommand},
    server_device::ADBServerDevice,
};

impl ADBServerDevice {
    /// Forward socket connection. Return the host endpoint listened on: `tcp:0` is resolved to the port allocated.
    pub fn forward(&self, remote: String, local: String) -> Result<String> {
        let transport = self.connect_to_device()?;
        transport.send_adb_request(&ADBCommand::Local(ADBLocalCommand::Forward(
            remote,
            local.clone(),
        )))?;
        if local != "tcp:0" {
            return Ok(local);
        }

        // The server answers the allocated port, as a length-prefixed string, after another `OKAY` on some versions
        let mut connection = transport.get_raw_connection()?;
        let mut buffer = [0; 4];
        connection.read_exact(&mut buffer)?;
        if &buffer == b"OKAY" {
            connection.read_exact(&mut buffer)?;
        }
        let length = usize::from_str_radix(std::str::from_utf8(&buffer)?, 16)?;
        let mut port = vec![0; length];
        connection.read_exact(&mut port)?;
        let port: u16 = std::str::from_utf8(&port)?.trim().parse().map_err(|_| {
            RustADBError::ADBRequestFailed(format!(
                "invalid allocated port {:?}",
                String::from_utf8_lossy(&port)
            ))
        })?;
        Ok(format!("tcp:{port}"))
    }

    /// Remove a previously applied forward rule by its local endpoint.
    pub fn forward_remove(&self, local: String) -> Result<()> {
        let transport = self.connect_to_device()?;

        transport
            .proxy_connection(
                &ADBCommand::Local(ADBLocalCommand::ForwardRemove(local)),
                false,
            )
            .map(|_| ())
    }

    /// Remove all previously applied forward rules
    pub fn forward_remove_all(&self) -> Result<()> {
        let transport = self.connect_to_device()?;

        transport
            .proxy_connection(&ADBCommand::Local(ADBLocalCommand::ForwardRemoveAll), false)
            .map(|_| ())
    }
}
