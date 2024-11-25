use rand::Rng;

use crate::{
    device::{adb_message_device::ADBMessageDevice, ADBTransportMessage, MessageCommand},
    ADBMessageTransport, RebootType, Result, RustADBError,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn reboot(&mut self, reboot_type: RebootType) -> Result<()> {
        let mut rng = rand::thread_rng();

        let message = ADBTransportMessage::new(
            MessageCommand::Open,
            rng.gen(), // Our 'local-id'
            0,
            format!("reboot:{}\0", reboot_type).as_bytes().to_vec(),
        );
        self.get_transport().write_message(message)?;

        let message = self.get_transport().read_message()?;

        if message.header().command() != MessageCommand::Okay {
            return Err(RustADBError::ADBShellNotSupported);
        }

        Ok(())
    }
}
