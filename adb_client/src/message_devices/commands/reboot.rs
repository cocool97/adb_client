use crate::{
    RebootType, Result,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
        message_commands::MessageCommand,
    },
    models::ADBLocalCommand,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn reboot(&mut self, reboot_type: RebootType) -> Result<()> {
        let mut session = self.open_session(&ADBLocalCommand::Reboot(reboot_type))?;

        session
            .read_message()
            .and_then(|message| message.assert_command(MessageCommand::Okay))
    }
}
