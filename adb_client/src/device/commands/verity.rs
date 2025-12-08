use crate::{
    ADBMessageTransport, Result,
    device::{MessageCommand, adb_message_device::ADBMessageDevice},
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn enable_verity(&mut self) -> Result<()> {
        self.open_session(format!("enable-verity:\0").as_bytes())?;

        self.get_transport_mut()
            .read_message()
            .and_then(|message| message.assert_command(MessageCommand::Okay))
    }

    pub(crate) fn disable_verity(&mut self) -> Result<()> {
        self.open_session(format!("disable-verity:\0").as_bytes())?;

        self.get_transport_mut()
            .read_message()
            .and_then(|message| message.assert_command(MessageCommand::Okay))
    }
}
