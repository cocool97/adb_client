use crate::{
    Result,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
    },
    models::ADBLocalCommand,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn root(&self) -> Result<()> {
        let mut session = self.open_session(&ADBLocalCommand::Root)?;

        // adbd restarts once its answer is sent: the connection may be lost before the session gets closed
        match session.recv_until_close() {
            Ok(output) => log::info!("{}", String::from_utf8_lossy(&output).trim()),
            Err(e) => log::debug!("connection lost while adbd restarts: {e}"),
        }

        Ok(())
    }
}
