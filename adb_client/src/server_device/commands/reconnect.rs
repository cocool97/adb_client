use crate::{
    Result,
    adb_transport::Connected,
    models::{ADBCommand, ADBLocalCommand},
    server_device::ADBServerDevice,
};

impl ADBServerDevice<Connected> {
    /// Reconnect device
    pub fn reconnect(&mut self) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .proxy_connection(&ADBCommand::Local(ADBLocalCommand::Reconnect), false)
            .map(|_| ())
    }
}
