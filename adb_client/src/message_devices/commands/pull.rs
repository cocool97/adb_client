use std::io::Write;

use crate::{
    Result, RustADBError,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
        message_commands::MessageSubcommand, utils::BinaryEncodable,
    },
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn pull<A: AsRef<str>, W: Write>(&self, source: A, output: W) -> Result<()> {
        let mut session = self.open_synchronization_session()?;
        let source = source.as_ref();

        let adb_stat_response = session.stat_with_explicit_ids(source)?;

        if adb_stat_response.file_perm == 0 {
            return Err(RustADBError::UnknownResponseType(
                "mode is 0: source file does not exist".to_string(),
            ));
        }

        let recv_buffer = MessageSubcommand::Recv.with_arg(u32::try_from(source.len())?);
        session.send_write_and_expect_okay(&recv_buffer.encode())?;
        session.send_write_and_expect_okay(source.as_bytes())?;

        session.recv_file(output)?;
        session.end_transaction()?;
        Ok(())
    }
}
