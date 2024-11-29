use crate::{models::ADBEmulatorCommand, ADBEmulatorDevice, Result};

impl ADBEmulatorDevice {
    /// Send a SMS to this emulator with given content with given phone number
    pub fn rotate(&mut self) -> Result<()> {
        let transport = self.connect()?;
        transport.send_command(ADBEmulatorCommand::Rotate)?;
        Ok(())
    }
}
