use std::io::Write;

use crate::{
    usb::{ADBUsbMessage, USBCommand},
    ADBDeviceExt, ADBUSBDevice, Result, RustADBError,
};

impl ADBDeviceExt for ADBUSBDevice {
    fn shell_command<S: ToString, W: Write>(
        &mut self,
        command: impl IntoIterator<Item = S>,
        mut output: W,
    ) -> Result<()> {
        let message = ADBUsbMessage::new(
            USBCommand::Open,
            1,
            0,
            format!(
                "shell:{}\0",
                command
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            )
            .as_bytes()
            .to_vec(),
        );
        self.transport.write_message(message)?;

        let response = self.transport.read_message()?;
        if response.command() != USBCommand::Okay {
            return Err(RustADBError::ADBRequestFailed(format!(
                "wrong command {}",
                response.command()
            )));
        }

        loop {
            let response = self.transport.read_message()?;
            if response.command() != USBCommand::Write {
                break;
            }

            let write = output.write(&response.into_payload())?;
        }

        Ok(())
    }
}
