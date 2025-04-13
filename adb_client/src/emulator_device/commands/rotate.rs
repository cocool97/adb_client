use crate::{ADBEmulatorDevice, Result, emulator_device::ADBEmulatorCommand};

impl ADBEmulatorDevice {
    /// Send a SMS to this emulator with given content with given phone number
    pub fn rotate(&mut self) -> Result<()> {
        self.connect()?.send_command(ADBEmulatorCommand::Rotate)
    }
}
