use adb_client::ADBEmulatorDevice;

use crate::models::{EmuCommand, EmulatorCommand};

pub fn handle_emulator_commands(emulator_command: EmulatorCommand) -> anyhow::Result<()> {
    let mut emulator = ADBEmulatorDevice::new(emulator_command.serial, None)?;

    match emulator_command.command {
        EmuCommand::Sms {
            phone_number,
            content,
        } => {
            emulator.send_sms(&phone_number, &content)?;
            log::info!("SMS sent to {phone_number}");
        }
        EmuCommand::Rotate => emulator.rotate()?,
    }

    Ok(())
}
