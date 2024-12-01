use rand::Rng;
use std::io::{Read, Write};

use crate::device::ShellMessageWriter;
use crate::Result;
use crate::{
    device::{ADBMessageDevice, ADBTransportMessage, MessageCommand},
    ADBMessageTransport, RustADBError,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    /// Runs 'command' in a shell on the device, and write its output and error streams into [`output`].
    pub(crate) fn shell_command<S: ToString, W: Write>(
        &mut self,
        command: impl IntoIterator<Item = S>,
        mut output: W,
    ) -> Result<()> {
        let message = ADBTransportMessage::new(
            MessageCommand::Open,
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
        self.get_transport_mut().write_message(message)?;

        let response = self.get_transport_mut().read_message()?;
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
    /// [W] has a 'static bound as it is internally used in a thread.
    pub(crate) fn shell<R: Read, W: Write + Send + 'static>(
        &mut self,
        mut reader: R,
        mut writer: W,
    ) -> Result<()> {
        let sync_directive = "shell:\0";

        let mut rng = rand::thread_rng();
        let message = ADBTransportMessage::new(
            MessageCommand::Open,
            rng.gen(), /* Our 'local-id' */
            0,
            sync_directive.into(),
        );
        let message = self.send_and_expect_okay(message)?;
        let local_id = message.header().arg1();
        let remote_id = message.header().arg0();

        let mut transport = self.get_transport().clone();

        // Reading thread, reads response from adbd
        std::thread::spawn(move || -> Result<()> {
            loop {
                let message = transport.read_message()?;

                // Acknowledge for more data
                let response =
                    ADBTransportMessage::new(MessageCommand::Okay, local_id, remote_id, vec![]);
                transport.write_message(response)?;

                match message.header().command() {
                    MessageCommand::Write => {}
                    MessageCommand::Okay => continue,
                    _ => return Err(RustADBError::ADBShellNotSupported),
                }

                writer.write_all(&message.into_payload())?;
                writer.flush()?;
            }
        });

        let transport = self.get_transport().clone();
        let mut shell_writer = ShellMessageWriter::new(transport, local_id, remote_id);

        // Read from given reader (that could be stdin e.g), and write content to device adbd
        if let Err(e) = std::io::copy(&mut reader, &mut shell_writer) {
            match e.kind() {
                std::io::ErrorKind::BrokenPipe => return Ok(()),
                _ => return Err(RustADBError::IOError(e)),
            }
        }

        Ok(())
    }
}
