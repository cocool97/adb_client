use crate::{
    models::{AdbServerCommand, SyncCommand},
    ADBServerDevice, Result,
};
use byteorder::{ByteOrder, LittleEndian};
use std::{
    io::{Read, Write},
    str,
};

impl ADBServerDevice {
    /// Lists files in [path] on the device.
    pub fn list<A: AsRef<str>>(&mut self, path: A) -> Result<()> {
        let serial = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;

        // Set device in SYNC mode
        self.get_transport_mut()
            .send_adb_request(AdbServerCommand::Sync)?;

        // Send a list command
        self.get_transport_mut()
            .send_sync_request(SyncCommand::List)?;

        self.handle_list_command(path)
    }

    // This command does not seem to work correctly. The devices I test it on just resturn
    // 'DONE' directly without listing anything.
    fn handle_list_command<S: AsRef<str>>(&mut self, path: S) -> Result<()> {
        let mut len_buf = [0_u8; 4];
        LittleEndian::write_u32(&mut len_buf, path.as_ref().len() as u32);

        // 4 bytes of command name is already sent by send_sync_request
        self.get_transport_mut()
            .get_raw_connection()?
            .write_all(&len_buf)?;

        // List send the string of the directory to list, and then the server send a list of files
        self.get_transport_mut()
            .get_raw_connection()?
            .write_all(path.as_ref().to_string().as_bytes())?;

        // Reads returned status code from ADB server
        let mut response = [0_u8; 4];
        loop {
            self.get_transport_mut()
                .get_raw_connection()?
                .read_exact(&mut response)?;
            match str::from_utf8(response.as_ref())? {
                "DENT" => {
                    // TODO: Move this to a struct that extract this data, but as the device
                    // I test this on does not return anything, I can't test it.
                    let mut file_mod = [0_u8; 4];
                    let mut file_size = [0_u8; 4];
                    let mut mod_time = [0_u8; 4];
                    let mut name_len = [0_u8; 4];

                    let mut connection = self.get_transport_mut().get_raw_connection()?;
                    connection.read_exact(&mut file_mod)?;
                    connection.read_exact(&mut file_size)?;
                    connection.read_exact(&mut mod_time)?;
                    connection.read_exact(&mut name_len)?;

                    let name_len = LittleEndian::read_u32(&name_len);
                    let mut name_buf = vec![0_u8; name_len as usize];
                    connection.read_exact(&mut name_buf)?;
                }
                "DONE" => {
                    return Ok(());
                }
                x => log::error!("Got an unknown response {}", x),
            }
        }
    }
}
