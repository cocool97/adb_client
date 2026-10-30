use crate::{
    Result,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
    },
    models::ADBLocalCommand,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn enable_verity(&self) -> Result<()> {
        self.verity(&ADBLocalCommand::EnableVerity)
    }

    pub(crate) fn disable_verity(&self) -> Result<()> {
        self.verity(&ADBLocalCommand::DisableVerity)
    }

    fn verity(&self, command: &ADBLocalCommand) -> Result<()> {
        let output = self.open_session(command)?.recv_until_close()?;
        log::info!("{}", String::from_utf8_lossy(&output).trim());
        Ok(())
    }
}
