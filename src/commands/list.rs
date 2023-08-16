use crate::{
    models::{AdbCommand, SyncCommand},
    AdbTcpConnection, Result,
};
use byteorder::{ByteOrder, LittleEndian};
use std::{
    io::{Read, Write},
    str,
};

impl AdbTcpConnection {
    /// Lists files in [path] on the device.
    pub fn list<S: ToString, A: AsRef<str>>(&mut self, serial: Option<S>, path: A) -> Result<()> {
        self.new_connection()?;

        match serial {
            None => self.send_adb_request(AdbCommand::TransportAny)?,
            Some(serial) => {
                self.send_adb_request(AdbCommand::TransportSerial(serial.to_string()))?
            }
        }

        // Set device in SYNC mode
        self.send_adb_request(AdbCommand::Sync)?;

        // Send a list command
        self.send_sync_request(SyncCommand::List(path.as_ref()))?;

        self.handle_list_command(path)
    }

    // This command does not seem to work correctly. The devices I test it on just resturn
    // 'DONE' directly without listing anything.
    fn handle_list_command<S: AsRef<str>>(&mut self, path: S) -> Result<()> {
        let mut len_buf = [0_u8; 4];
        LittleEndian::write_u32(&mut len_buf, path.as_ref().len() as u32);

        // 4 bytes of command name is already sent by send_sync_request
        self.tcp_stream.write_all(&len_buf)?;

        // List sends the string of the directory to list, and then the server sends a list of files
        self.tcp_stream
            .write_all(path.as_ref().to_string().as_bytes())?;

        // Reads returned status code from ADB server
        let mut response = [0_u8; 4];
        loop {
            self.tcp_stream.read_exact(&mut response)?;
            match str::from_utf8(response.as_ref())? {
                "DENT" => {
                    // TODO: Move this to a struct that extract this data, but as the device
                    // I test this on does not return anything, I can't test it.
                    let mut file_mod = [0_u8; 4];
                    let mut file_size = [0_u8; 4];
                    let mut mod_time = [0_u8; 4];
                    let mut name_len = [0_u8; 4];
                    self.tcp_stream.read_exact(&mut file_mod)?;
                    self.tcp_stream.read_exact(&mut file_size)?;
                    self.tcp_stream.read_exact(&mut mod_time)?;
                    self.tcp_stream.read_exact(&mut name_len)?;
                    let name_len = LittleEndian::read_u32(&name_len);
                    let mut name_buf = vec![0_u8; name_len as usize];
                    self.tcp_stream.read_exact(&mut name_buf)?;
                }
                "DONE" => {
                    return Ok(());
                }
                x => println!("Unknown response {}", x),
            }
        }
    }
}
