use std::io::Write;

use crate::{
    ADBMessageTransport, Result, RustADBError,
    device::{
        ADBTransportMessage, MessageCommand, adb_message_device::ADBMessageDevice,
        models::MessageSubcommand,
    },
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn pull<A: AsRef<str>, W: Write>(&mut self, source: A, output: W) -> Result<()> {
        self.begin_synchronization()?;
        let source = source.as_ref();

        let adb_stat_response = self.stat_with_explicit_ids(source)?;

        if adb_stat_response.file_perm == 0 {
            return Err(RustADBError::UnknownResponseType(
                "mode is 0: source file does not exist".to_string(),
            ));
        }

        let local_id = self.get_local_id()?;
        let remote_id = self.get_remote_id()?;

        self.get_transport_mut().write_message_with_timeout(
            ADBTransportMessage::new(MessageCommand::Okay, local_id, remote_id, &[]),
            std::time::Duration::from_secs(4),
        )?;

        let recv_buffer = MessageSubcommand::Recv.with_arg(source.len() as u32);
        let recv_buffer =
            bincode::serialize(&recv_buffer).map_err(|_e| RustADBError::ConversionError)?;
        self.send_and_expect_okay(ADBTransportMessage::new(
            MessageCommand::Write,
            self.get_local_id()?,
            self.get_remote_id()?,
            &recv_buffer,
        ))?;
        self.send_and_expect_okay(ADBTransportMessage::new(
            MessageCommand::Write,
            self.get_local_id()?,
            self.get_remote_id()?,
            source.as_bytes(),
        ))?;

        self.recv_file(output)?;
        self.end_transaction()?;
        Ok(())
    }
}
