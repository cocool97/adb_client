use crate::{ADBEmulatorDevice, Result, emulator_device::ADBEmulatorCommand};

impl ADBEmulatorDevice {
    /// Send a SMS to this emulator with given content with given phone number
    pub fn send_sms(&mut self, phone_number: &str, content: &str) -> Result<()> {
        self.connect()?.send_command(ADBEmulatorCommand::Sms(
            phone_number.to_string(),
            content.to_string(),
        ))
    }
}
