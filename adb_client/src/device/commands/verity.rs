use crate::{
    ADBMessageTransport, Result,
    device::{MessageCommand, adb_message_device::ADBMessageDevice},
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn enable_verity(&mut self) -> Result<()> {
        self.open_session(b"enable-verity:\0")?;

        self.get_transport_mut()
            .read_message()
            .and_then(|message| message.assert_command(MessageCommand::Okay))
    }

    pub(crate) fn disable_verity(&mut self) -> Result<()> {
        self.open_session(b"disable-verity:\0")?;

        self.get_transport_mut()
            .read_message()
            .and_then(|message| message.assert_command(MessageCommand::Okay))
    }
}
