use crate::{
    RebootType, Result,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
        message_commands::MessageCommand,
    },
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn reboot(&mut self, reboot_type: RebootType) -> Result<()> {
        self.open_session(format!("reboot:{reboot_type}\0").as_bytes())?;

        self.get_transport_mut()
            .read_message()
            .and_then(|message| message.assert_command(MessageCommand::Okay))
    }
}
