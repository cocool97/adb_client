use std::io::{Read, Write};

use rand::Rng;

use crate::{
    usb::{ADBUsbMessage, USBCommand, USBSubcommand}, ADBDeviceExt, ADBUSBDevice, FileStat, RebootType, Result, RustADBError
};

use super::USBShellWriter;

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
        if response.header().command() != USBCommand::Okay {
            return Err(RustADBError::ADBRequestFailed(format!(
                "wrong command {}",
                response.header().command()
            )));
        }

        loop {
            let response = self.transport.read_message()?;
            if response.header().command() != USBCommand::Write {
                break;
            }

            output.write_all(&response.into_payload())?;
        }

        Ok(())
    }

    fn shell<R: Read, W: Write + Send + 'static>(
        &mut self,
        mut reader: R,
        mut writer: W,
    ) -> Result<()> {
        let mut rng = rand::thread_rng();

        let message = ADBUsbMessage::new(
            USBCommand::Open,
            rng.gen(), // Our 'local-id'
            0,
            "shell:\0".as_bytes().to_vec(),
        );
        self.transport.write_message(message)?;

        let message = self.transport.read_message()?;

        if message.header().command() != USBCommand::Okay {
            return Err(RustADBError::ADBShellNotSupported);
        }

        // We get identifiers here when received from adbd
        // As this is received frame, our remote-id is adbd local-id and our local-id is adbd remote-id
        let remote_id = message.header().arg0();
        let local_id = message.header().arg1();

        let mut transport = self.transport.clone();

        // Reading thread, reads response from adbd
        std::thread::spawn(move || -> Result<()> {
            loop {
                let message = transport.read_message()?;

                // Acknowledge for more data
                let response = ADBUsbMessage::new(USBCommand::Okay, local_id, remote_id, vec![]);
                transport.write_message(response)?;

                match message.header().command() {
                    USBCommand::Write => {}
                    USBCommand::Okay => continue,
                    _ => return Err(RustADBError::ADBShellNotSupported),
                }

                writer.write_all(&message.into_payload())?;
                writer.flush()?;
            }
        });

        let mut shell_writer = USBShellWriter::new(self.transport.clone(), local_id, remote_id);

        // Read from given reader (that could be stdin e.g), and write content to device adbd
        if let Err(e) = std::io::copy(&mut reader, &mut shell_writer) {
            match e.kind() {
                std::io::ErrorKind::BrokenPipe => return Ok(()),
                _ => return Err(RustADBError::IOError(e)),
            }
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

    fn reboot(&mut self, reboot_type: RebootType) -> Result<()> {
        let mut rng = rand::thread_rng();

        let message = ADBUsbMessage::new(
            USBCommand::Open,
            rng.gen(), // Our 'local-id'
            0,
            format!("reboot:{}\0", reboot_type).as_bytes().to_vec(),
        );
        self.transport.write_message(message)?;

        let message = self.transport.read_message()?;

        if message.header().command() != USBCommand::Okay {
            return Err(RustADBError::ADBShellNotSupported);
        }

        Ok(())
    }
}