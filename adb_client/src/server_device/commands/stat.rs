use std::io::{Read, Write};

use byteorder::{ByteOrder, LittleEndian};

use crate::{
    models::{AdbServerCommand, AdbStatResponse, SyncCommand},
    ADBServerDevice, Result, RustADBError,
};

impl ADBServerDevice {
    fn handle_stat_command<S: AsRef<str>>(&mut self, path: S) -> Result<AdbStatResponse> {
        let mut len_buf = [0_u8; 4];
        LittleEndian::write_u32(&mut len_buf, path.as_ref().len() as u32);

        // 4 bytes of command name is already sent by send_sync_request
        self.get_transport_mut()
            .get_raw_connection()?
            .write_all(&len_buf)?;
        self.get_transport_mut()
            .get_raw_connection()?
            .write_all(path.as_ref().to_string().as_bytes())?;

        // Reads returned status code from ADB server
        let mut response = [0_u8; 4];
        self.get_transport_mut()
            .get_raw_connection()?
            .read_exact(&mut response)?;
        match std::str::from_utf8(response.as_ref())? {
            "STAT" => {
                let mut data = [0_u8; 12];
                self.get_transport_mut()
                    .get_raw_connection()?
                    .read_exact(&mut data)?;

                Ok(data.into())
            }
            x => Err(RustADBError::UnknownResponseType(format!(
                "Unknown response {}",
                x
            ))),
        }
    }

    /// Stat file given as path on the device.
    pub fn stat<A: AsRef<str>>(&mut self, path: A) -> Result<AdbStatResponse> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;

        // Set device in SYNC mode
        self.get_transport_mut()
            .send_adb_request(AdbServerCommand::Sync)?;

        // Send a "Stat" command
        self.get_transport_mut()
            .send_sync_request(SyncCommand::Stat)?;

        self.handle_stat_command(path)
    }
}
