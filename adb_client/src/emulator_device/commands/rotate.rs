use crate::{emulator_device::ADBEmulatorCommand, ADBEmulatorDevice, Result};

impl ADBEmulatorDevice {
    /// Send a SMS to this emulator with given content with given phone number
    pub fn rotate(&mut self) -> Result<()> {
        self.connect()?.send_command(ADBEmulatorCommand::Rotate)
    }
}
