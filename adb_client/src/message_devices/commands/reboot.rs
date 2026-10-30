use crate::{
    RebootType, Result, RustADBError,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
    },
    models::ADBLocalCommand,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn reboot(&self, reboot_type: RebootType) -> Result<()> {
        let mut session = self.open_session(&ADBLocalCommand::Reboot(reboot_type))?;

        // adbd only answers when the reboot failed, the connection is lost otherwise
        match session.recv_until_close() {
            Ok(output) if output.is_empty() => Ok(()),
            Ok(output) => Err(RustADBError::ADBRequestFailed(
                String::from_utf8_lossy(&output).trim().to_string(),
            )),
            Err(e) => {
                log::debug!("connection lost while device reboots: {e}");
                Ok(())
            }
        }
    }
}
