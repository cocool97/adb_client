use std::io::Write;

use crate::{
    device::{
        adb_message_device::ADBMessageDevice, models::MessageSubcommand, ADBTransportMessage,
        MessageCommand,
    },
    ADBMessageTransport, Result, RustADBError,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn pull<A: AsRef<str>, W: Write>(&mut self, source: A, output: W) -> Result<()> {
        let (local_id, remote_id) = self.begin_synchronization()?;
        let source = source.as_ref();

        let adb_stat_response = self.stat_with_explicit_ids(source, local_id, remote_id)?;
        self.get_transport_mut()
            .write_message(ADBTransportMessage::new(
                MessageCommand::Okay,
                local_id,
                remote_id,
                "".into(),
            ))?;

        log::debug!("{:?}", adb_stat_response);
        if adb_stat_response.file_perm == 0 {
            return Err(RustADBError::UnknownResponseType(
                "mode is 0: source file does not exist".to_string(),
            ));
        }

        let recv_buffer = MessageSubcommand::Recv.with_arg(source.len() as u32);
        let recv_buffer =
            bincode::serialize(&recv_buffer).map_err(|_e| RustADBError::ConversionError)?;
        self.send_and_expect_okay(ADBTransportMessage::new(
            MessageCommand::Write,
            local_id,
            remote_id,
            recv_buffer,
        ))?;
        self.send_and_expect_okay(ADBTransportMessage::new(
            MessageCommand::Write,
            local_id,
            remote_id,
            source.into(),
        ))?;

        self.recv_file(local_id, remote_id, output)?;
        self.end_transaction(local_id, remote_id)?;
        Ok(())
    }
}
