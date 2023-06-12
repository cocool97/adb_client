use crate::{
    models::{AdbCommand, SyncCommand},
    AdbTcpConnexion, Result,
};

impl AdbTcpConnexion {
    /// Lists files in [path] on the device.
    pub fn list<S: ToString>(&mut self, serial: Option<S>, path: S) -> Result<()> {
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

        // Send a list command
        self.send_sync_request(SyncCommand::List(&path.to_string()))?;

        Ok(())
    }
}
