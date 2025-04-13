use std::io::{ErrorKind, Read, Write};

use crate::Result;
use crate::device::ShellMessageWriter;
use crate::{
    ADBMessageTransport, RustADBError,
    device::{ADBMessageDevice, ADBTransportMessage, MessageCommand},
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    /// Runs 'command' in a shell on the device, and write its output and error streams into output.
    pub(crate) fn shell_command(&mut self, command: &[&str], output: &mut dyn Write) -> Result<()> {
        let response = self.open_session(format!("shell:{}\0", command.join(" "),).as_bytes())?;

        if response.header().command() != MessageCommand::Okay {
            return Err(RustADBError::ADBRequestFailed(format!(
                "wrong command {}",
                response.header().command()
            )));
        }

        loop {
            let response = self.get_transport_mut().read_message()?;
            if response.header().command() != MessageCommand::Write {
                break;
            }

            output.write_all(&response.into_payload())?;
        }

        Ok(())
    }

    /// Starts an interactive shell session on the device.
    /// Input data is read from [reader] and write to [writer].
    pub(crate) fn shell(
        &mut self,
        mut reader: &mut dyn Read,
        mut writer: Box<(dyn Write + Send)>,
    ) -> Result<()> {
        self.open_session(b"shell:\0")?;

        let mut transport = self.get_transport().clone();

        let local_id = self.get_local_id()?;
        let remote_id = self.get_remote_id()?;

        // Reading thread, reads response from adbd
        std::thread::spawn(move || -> Result<()> {
            loop {
                let message = transport.read_message()?;

                // Acknowledge for more data
                let response =
                    ADBTransportMessage::new(MessageCommand::Okay, local_id, remote_id, &[]);
                transport.write_message(response)?;

                match message.header().command() {
                    MessageCommand::Write => {
                        writer.write_all(&message.into_payload())?;
                        writer.flush()?;
                    }
                    MessageCommand::Okay => continue,
                    _ => return Err(RustADBError::ADBShellNotSupported),
                }
            }
        });

        let transport = self.get_transport().clone();
        let mut shell_writer = ShellMessageWriter::new(transport, local_id, remote_id);

        // Read from given reader (that could be stdin e.g), and write content to device adbd
        if let Err(e) = std::io::copy(&mut reader, &mut shell_writer) {
            match e.kind() {
                ErrorKind::BrokenPipe => return Ok(()),
                _ => return Err(RustADBError::IOError(e)),
            }
        }

        Ok(())
    }
}
