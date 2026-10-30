use crate::{
    Result,
    models::{ADBCommand, ADBLocalCommand, RemountInfo},
    server_device::ADBServerDevice,
};
use std::io::Read;

impl ADBServerDevice {
    /// Remounts the device filesystem as read-write
    pub fn remount(&self) -> Result<Vec<RemountInfo>> {
        let transport = self.connect_to_device()?;

        transport.send_adb_request(&ADBCommand::Local(ADBLocalCommand::Remount))?;

        let mut data = [0; 1024];
        let read_amount = transport.get_raw_connection()?.read(&mut data)?;

        let response = String::from_utf8_lossy(&data[0..read_amount]);
        RemountInfo::from_str_response(&response)
    }
}
