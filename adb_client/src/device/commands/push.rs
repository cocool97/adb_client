use std::io::Read;

use crate::{
    ADBMessageTransport, Result,
    device::{
        ADBTransportMessage, MessageCommand, MessageSubcommand,
        adb_message_device::{self, ADBMessageDevice},
    },
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn push<R: Read, A: AsRef<str>>(&mut self, stream: R, path: A) -> Result<()> {
        self.begin_synchronization()?;

        let path_header = format!("{},0777", path.as_ref());

        let send_buffer = MessageSubcommand::Send.with_arg(u32::try_from(path_header.len())?);
        let mut send_buffer = adb_message_device::bincode_serialize_to_vec(&send_buffer)?;
        send_buffer.append(&mut path_header.as_bytes().to_vec());

        self.send_and_expect_okay(ADBTransportMessage::new(
            MessageCommand::Write,
            self.get_local_id()?,
            self.get_remote_id()?,
            &send_buffer,
        ))?;

        self.push_file(self.get_local_id()?, self.get_remote_id()?, stream)?;

        self.end_transaction()?;

        Ok(())
    }
}
