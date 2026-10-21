use std::{
    io::{ErrorKind, Read, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

use crate::models::ADBLocalCommand;
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
    pub(crate) fn shell_command(
        &mut self,
        command: &dyn AsRef<str>,
        mut stdout: Option<&mut dyn Write>,
        _stderr: Option<&mut dyn Write>,
    ) -> Result<Option<u8>> {
        let mut session = self.open_session(&ADBLocalCommand::ShellCommand(
            command.as_ref().to_string(),
            Vec::new(),
        ))?;

        loop {
            let message = session.recv_and_reply_okay()?;
            if message.header().command() == MessageCommand::Clse {
                break;
            }
            // should this just write for ::Write messages?
            if let Some(ref mut stdout) = stdout {
                stdout.write_all(&message.into_payload())?;
            }
        }

        Ok(None)
    }

    /// Starts an interactive shell session on the device.
    /// Input data is read from [reader] and write to [writer].
    pub(crate) fn shell(
        &mut self,
        reader: Box<dyn Read + Send>,
        writer: Box<dyn Write + Send>,
    ) -> Result<()> {
        self.bidirectional_session(&ADBLocalCommand::Shell, reader, writer)
    }

    /// Runs `command` on the device.
    /// Input data is read from [reader] and write to [writer].
    pub(crate) fn exec(
        &mut self,
        command: &str,
        reader: Box<dyn Read + Send>,
        writer: Box<dyn Write + Send>,
    ) -> Result<()> {
        self.bidirectional_session(&ADBLocalCommand::Exec(command.to_string()), reader, writer)
    }

    /// Starts a bidirectional (interactive) session. This can be a shell or an exec session.
    ///
    /// Data read from `reader` is forwarded to the device from a background thread, while device output
    /// is written to `writer` from the current thread. The session ends once the stream is closed by the device
    /// or when `reader` reaches EOF or fails.
    fn bidirectional_session(
        &mut self,
        local_command: &ADBLocalCommand,
        mut reader: Box<dyn Read + Send>,
        mut writer: Box<dyn Write + Send>,
    ) -> Result<()> {
        let session = self.open_session(local_command)?;

        let local_id = session.local_id();
        let remote_id = session.remote_id();
        let stream_closed = Arc::new(AtomicBool::new(false));

        // Forward `reader` (that could be stdin e.g) to device adbd.
        // This thread may stay blocked on `reader` forever, so it is never joined.
        let (local_result_tx, local_result_rx) = mpsc::channel();
        let mut shell_writer = ShellMessageWriter::new(
            self.get_transport_mut().clone(),
            local_id,
            remote_id,
            session.max_payload_size(),
            stream_closed.clone(),
        );
        std::thread::spawn(move || {
            let result = match std::io::copy(&mut reader, &mut shell_writer) {
                Ok(_) => Ok(()),
                // Stream has been closed by remote, nothing more to do
                Err(e) if e.kind() == ErrorKind::BrokenPipe => Ok(()),
                Err(e) => Err(RustADBError::IOError(e)),
            };

            // best-effort error shell closing & error reporting
            match result {
                Ok(()) => {
                    let _ = shell_writer.close();
                }
                Err(err) => {
                    let _ = local_result_tx.send(err);
                }
            }
        });

        let mut transport = self.get_transport_mut().clone();
        let remote_result = Self::forward_data_from_remote_adbd(
            &mut transport,
            local_id,
            remote_id,
            &stream_closed,
            &mut writer,
        );

        if remote_result.is_err() && !stream_closed.swap(true, Ordering::SeqCst) {
            // Best effort, tell adbd we are not listening anymore
            let _ = transport.write_message(ADBTransportMessage::try_new(
                MessageCommand::Clse,
                local_id,
                remote_id,
                &[],
            )?);
        }

        remote_result?;

        // Input thread may still be blocked on `reader`, only report an already known error
        match local_result_rx.try_recv() {
            Ok(e) => Err(e),
            _ => Ok(()),
        }
    }

    /// Write data received from adbd into the given `writer` until the stream gets closed.
    fn forward_data_from_remote_adbd(
        transport: &mut T,
        local_id: u32,
        remote_id: u32,
        closed: &AtomicBool,
        writer: &mut dyn Write,
    ) -> Result<()> {
        loop {
            let message = transport.read_message()?;

            match message.header().command() {
                MessageCommand::Write => {
                    writer.write_all(message.payload())?;
                    writer.flush()?;

                    // Acknowledge for more data
                    transport.write_message(ADBTransportMessage::try_new(
                        MessageCommand::Okay,
                        local_id,
                        remote_id,
                        &[],
                    )?)?;
                }
                // Acknowledgement of data we sent
                MessageCommand::Okay => {}
                MessageCommand::Clse => {
                    // Acknowledge the close if it was initiated by adbd
                    if !closed.swap(true, Ordering::SeqCst) {
                        transport.write_message(ADBTransportMessage::try_new(
                            MessageCommand::Clse,
                            local_id,
                            remote_id,
                            &[],
                        )?)?;
                    }
                    return Ok(());
                }
                c => {
                    return Err(RustADBError::ADBRequestFailed(format!(
                        "unexpected {c} message received during session"
                    )));
                }
            }
        }
    }
}
