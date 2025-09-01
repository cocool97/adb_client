use std::io::Read;

use crate::{
    Result,
    message_devices::{
        adb_message_device::{ADBMessageDevice, bincode_serialize_to_vec},
        adb_message_transport::ADBMessageTransport,
        adb_transport_message::ADBTransportMessage,
        message_commands::{MessageCommand, MessageSubcommand},
    },
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn push<R: Read, A: AsRef<str>>(&mut self, stream: R, path: A) -> Result<()> {
        let session = self.begin_synchronization()?;

        let path_header = format!("{},0777", path.as_ref());

        let send_buffer = MessageSubcommand::Send.with_arg(u32::try_from(path_header.len())?);
        let mut send_buffer = bincode_serialize_to_vec(&send_buffer)?;
        send_buffer.append(&mut path_header.as_bytes().to_vec());

        self.send_and_expect_okay(ADBTransportMessage::try_new(
            MessageCommand::Write,
            session.local_id(),
            session.remote_id(),
            &send_buffer,
        )?)?;

        self.push_file(session, stream)?;
        self.end_transaction(session)?;

        Ok(())
    }
}
