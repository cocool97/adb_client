use std::io::Read;

use crate::{
    device::{
        adb_message_device::ADBMessageDevice, ADBTransportMessage, MessageCommand,
        MessageSubcommand,
    },
    ADBMessageTransport, Result, RustADBError,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn push<R: Read, A: AsRef<str>>(&mut self, stream: R, path: A) -> Result<()> {
        let (local_id, remote_id) = self.begin_synchronization()?;

        let path_header = format!("{},0777", path.as_ref());

        let send_buffer = MessageSubcommand::Send.with_arg(path_header.len() as u32);
        let mut send_buffer =
            bincode::serialize(&send_buffer).map_err(|_e| RustADBError::ConversionError)?;
        send_buffer.append(&mut path_header.as_bytes().to_vec());

        self.send_and_expect_okay(ADBTransportMessage::new(
            MessageCommand::Write,
            local_id,
            remote_id,
            send_buffer,
        ))?;

        self.push_file(local_id, remote_id, stream)?;

        self.end_transaction(local_id, remote_id)?;

        Ok(())
    }
}
