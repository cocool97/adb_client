use crate::{
    models::{AdbCommand, SyncCommand},
    AdbTcpConnexion, Result,
};

impl AdbTcpConnexion {
    /// Stat file given as [path] on the device.
    pub fn stat<S: ToString, A: AsRef<str>>(&mut self, serial: Option<S>, path: A) -> Result<()> {
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
        self.send_sync_request(SyncCommand::Stat(path.as_ref()))?;

        Ok(())
    }
}
