use crate::{
    models::{AdbCommand, SyncCommand},
    AdbTcpConnexion, Result,
};

impl AdbTcpConnexion {
    /// Pulls [path] to [filename] from the device.
    pub fn pull<S: ToString>(&mut self, serial: Option<S>, path: S, filename: S) -> Result<()> {
        self.new_connection()?;

        match serial {
            None => Self::send_adb_request(&mut self.tcp_stream, AdbCommand::TransportAny)?,
            Some(serial) => Self::send_adb_request(
                &mut self.tcp_stream,
                AdbCommand::TransportSerial(serial.to_string()),
            )?,
        }

        // Set device in SYNC mode
        Self::send_adb_request(&mut self.tcp_stream, AdbCommand::Sync)?;

        // Send a recv command
        self.send_sync_request(SyncCommand::Recv(&path.to_string(), filename.to_string()))?;

        Ok(())
    }
}
