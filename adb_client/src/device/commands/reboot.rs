use crate::{
    device::{adb_message_device::ADBMessageDevice, MessageCommand},
    ADBMessageTransport, RebootType, Result, RustADBError,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn reboot(&mut self, reboot_type: RebootType) -> Result<()> {
        self.open_session(format!("reboot:{}\0", reboot_type).as_bytes())?;

        let message = self.get_transport_mut().read_message()?;

        if message.header().command() != MessageCommand::Okay {
            return Err(RustADBError::ADBShellNotSupported);
        }

        Ok(())
    }
}
