use crate::{
    models::AdbStatResponse,
    usb::{ADBUsbMessage, USBCommand, USBSubcommand},
    ADBDeviceExt, ADBUSBDevice, RebootType, Result, RustADBError,
};
use rand::Rng;
use std::io::{Read, Write};

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
        let sync_directive = "shell:\0";

        let mut rng = rand::thread_rng();
        let message = ADBUsbMessage::new(
            USBCommand::Open,
            rng.gen(), /* Our 'local-id' */
            0,
            sync_directive.into(),
        );
        let message = self.send_and_expect_okay(message)?;
        let local_id = message.header().arg1();
        let remote_id = message.header().arg0();

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

    fn stat(&mut self, remote_path: &str) -> Result<AdbStatResponse> {
        let (local_id, remote_id) = self.begin_synchronization()?;
        let adb_stat_response = self.stat_with_explicit_ids(remote_path, local_id, remote_id)?;
        self.end_transaction(local_id, remote_id)?;
        Ok(adb_stat_response)
    }

    fn pull<A: AsRef<str>, W: Write>(&mut self, source: A, output: W) -> Result<()> {
        let (local_id, remote_id) = self.begin_synchronization()?;
        let source = source.as_ref();

        let adb_stat_response = self.stat_with_explicit_ids(source, local_id, remote_id)?;
        self.transport.write_message(ADBUsbMessage::new(
            USBCommand::Okay,
            local_id,
            remote_id,
            "".into(),
        ))?;

        log::debug!("{:?}", adb_stat_response);
        if adb_stat_response.file_perm == 0 {
            return Err(RustADBError::UnknownResponseType(
                "mode is 0: source file does not exist".to_string(),
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

        self.recv_file(local_id, remote_id, output)?;
        self.end_transaction(local_id, remote_id)?;
        Ok(())
    }

    fn push<R: Read, A: AsRef<str>>(&mut self, stream: R, path: A) -> Result<()> {
        let (local_id, remote_id) = self.begin_synchronization()?;

        let path_header = format!("{},0777", path.as_ref());

        let send_buffer = USBSubcommand::Send.with_arg(path_header.len() as u32);
        let mut send_buffer =
            bincode::serialize(&send_buffer).map_err(|_e| RustADBError::ConversionError)?;
        send_buffer.append(&mut path_header.as_bytes().to_vec());

        self.send_and_expect_okay(ADBUsbMessage::new(
            USBCommand::Write,
            local_id,
            remote_id,
            send_buffer,
        ))?;

        self.push_file(local_id, remote_id, stream)?;

        self.end_transaction(local_id, remote_id)?;

        Ok(())
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
