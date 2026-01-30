use adb_client::emulator::ADBEmulatorDevice;

use crate::models::{ADBCliResult, EmuCommand, EmulatorCommand};

pub fn handle_emulator_commands(emulator_command: EmulatorCommand) -> ADBCliResult<()> {
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
        EmuCommand::AvdDiscoveryPath => {
            let path = emulator.avd_discovery_path()?;
            log::info!("AVD discovery path: {}", path.display());
            println!("{}", path.display());
        }
        EmuCommand::AvdGrpcPort => {
            let port = emulator.avd_grpc_port()?;
            log::info!("gRPC port: {port}");
            println!("{port}");
        }
        EmuCommand::Raw { command } => {
            let response = emulator.send_raw_command(&command)?;
            println!("{response}");
        }
    }

    Ok(())
}
