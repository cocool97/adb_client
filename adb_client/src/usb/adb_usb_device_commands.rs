use std::io::Write;

use crate::{
    usb::{ADBUsbMessage, SubcommandWithArg, USBCommand, USBSubcommand},
    ADBDeviceExt, ADBUSBDevice, FileStat, Result, RustADBError,
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

    // stat a file
    fn stat(&mut self, remote_path: &str, local_id: u32, remote_id: u32) -> Result<FileStat> {
        let stat_buffer = USBSubcommand::Stat.with_arg(remote_path.len() as u32);
        let message = ADBUsbMessage::new(
            USBCommand::Write,
            local_id,
            remote_id,
            bincode::serialize(&stat_buffer).map_err(|_e| RustADBError::ConversionError)?,
        );
        self.send_and_expect_okay(message)?;
        self.send_and_expect_okay(ADBUsbMessage::new(
            USBCommand::Write,
            local_id,
            remote_id,
            remote_path.into(),
        ))?;
        let response = self.recv_and_reply_okay(local_id, remote_id)?;
        bincode::deserialize(&response.into_payload()).map_err(|_e| RustADBError::ConversionError)
    }

    fn pull<W: Write>(&mut self, source: &str, output: W) -> Result<()> {
        let sync_directive = "sync:.\0";

        let message = ADBUsbMessage::new(USBCommand::Open, 12345, 0, sync_directive.into());
        let message = self.send_and_expect_okay(message)?;
        let local_id = message.arg1();
        let remote_id = message.arg0();

        let FileStat { mode, file_size } = self.stat(source, local_id, remote_id)?;

        log::debug!("mode: {}, file size: {}", mode, file_size);
        if mode == 0 {
            return Err(RustADBError::UnknownResponseType(
                "mode is 0: source apk does not exist".to_string(),
            ));
        }

        let recv_buffer = USBSubcommand::Recv.with_arg(source.len() as u32);
        let recv_buffer =
            bincode::serialize(&recv_buffer).map_err(|_e| RustADBError::ConversionError)?;
        self.send_and_expect_okay(ADBUsbMessage::new(
            USBCommand::Write,
            local_id,
            remote_id,
            recv_buffer,
        ))?;
        self.send_and_expect_okay(ADBUsbMessage::new(
            USBCommand::Write,
            local_id,
            remote_id,
            source.into(),
        ))?;

        self.recv_file(local_id, remote_id, output)
    }
}
