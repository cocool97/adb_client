use std::io::{ErrorKind, Read, Write};
use std::time::Duration;

use crate::{
    Result, RustADBError,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
        adb_transport_message::ADBTransportMessage, commands::utils::ShellMessageWriter,
        message_commands::MessageCommand,
    },
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    /// Runs 'command' in a shell on the device, and write its output and error streams into output.
    pub(crate) fn shell_command(&mut self, command: &[&str], output: &mut dyn Write) -> Result<()> {
        let session = self.open_session(format!("shell:{}\0", command.join(" "),).as_bytes())?;

        let mut transport = self.get_transport().clone();

        loop {
            let message = transport.read_message()?;
            let command = message.header().command();

            if command == MessageCommand::Clse {
                break;
            }

            self.get_transport_mut()
                .write_message(ADBTransportMessage::try_new(
                    MessageCommand::Okay,
                    session.local_id(),
                    session.remote_id(),
                    &[],
                )?)?;

            output.write_all(&message.into_payload())?;
        }

        // some devices will repeat the trailing CLSE command to ensure
        // the client has acknowledged it. Read them quickly if present.
        while let Ok(_discard_close_message) =
            transport.read_message_with_timeout(Duration::from_millis(20))
        {}

        Ok(())
    }

    /// Starts an interactive shell session on the device.
    /// Input data is read from [reader] and write to [writer].
    pub(crate) fn shell(
        &mut self,
        mut reader: &mut dyn Read,
        mut writer: Box<dyn Write + Send>,
    ) -> Result<()> {
        let session = self.open_session(b"shell:\0")?;

        let mut transport = self.get_transport().clone();

        // Reading thread, reads response from adbd
        std::thread::spawn(move || -> Result<()> {
            loop {
                let message = transport.read_message()?;

                // Acknowledge for more data
                let response = ADBTransportMessage::try_new(
                    MessageCommand::Okay,
                    session.local_id(),
                    session.remote_id(),
                    &[],
                )?;
                transport.write_message(response)?;

                match message.header().command() {
                    MessageCommand::Write => {
                        writer.write_all(&message.into_payload())?;
                        writer.flush()?;
                    }
                    MessageCommand::Okay => {}
                    _ => return Err(RustADBError::ADBShellNotSupported),
                }
            }
        });

        let transport = self.get_transport().clone();
        let mut shell_writer =
            ShellMessageWriter::new(transport, session.local_id(), session.remote_id());

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
