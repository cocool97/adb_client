use byteorder::{LittleEndian, ReadBytesExt};
use rand::Rng;
use std::io::{Cursor, Read, Seek};

use crate::{constants::BUFFER_SIZE, ADBMessageTransport, AdbStatResponse, Result, RustADBError};

use super::{models::MessageSubcommand, ADBTransportMessage, MessageCommand};

/// Generic structure representing an ADB device reachable over an [`ADBMessageTransport`].
/// Structure is totally agnostic over which transport is truly used.
#[derive(Debug)]
pub struct ADBMessageDevice<T: ADBMessageTransport> {
    transport: T,
}

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    /// Instantiate a new [`ADBMessageTransport`]
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub(crate) fn get_transport(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Receive a message and acknowledge it by replying with an `OKAY` command
    pub(crate) fn recv_and_reply_okay(
        &mut self,
        local_id: u32,
        remote_id: u32,
    ) -> Result<ADBTransportMessage> {
        let message = self.transport.read_message()?;
        self.transport.write_message(ADBTransportMessage::new(
            MessageCommand::Okay,
            local_id,
            remote_id,
            "".into(),
        ))?;
        Ok(message)
    }

    /// Expect a message with an `OKAY` command after sending a message.
    pub(crate) fn send_and_expect_okay(
        &mut self,
        message: ADBTransportMessage,
    ) -> Result<ADBTransportMessage> {
        self.transport.write_message(message)?;
        let message = self.transport.read_message()?;
        let received_command = message.header().command();
        if received_command != MessageCommand::Okay {
            return Err(RustADBError::ADBRequestFailed(format!(
                "expected command OKAY after message, got {}",
                received_command
            )));
        }
        Ok(message)
    }

    pub(crate) fn recv_file<W: std::io::Write>(
        &mut self,
        local_id: u32,
        remote_id: u32,
        mut output: W,
    ) -> std::result::Result<(), RustADBError> {
        let mut len: Option<u64> = None;
        loop {
            let payload = self
                .recv_and_reply_okay(local_id, remote_id)?
                .into_payload();
            let mut rdr = Cursor::new(&payload);
            while rdr.position() != payload.len() as u64 {
                match len.take() {
                    Some(0) | None => {
                        rdr.seek_relative(4)?;
                        len.replace(rdr.read_u32::<LittleEndian>()? as u64);
                    }
                    Some(length) => {
                        log::debug!("len = {length}");
                        let remaining_bytes = payload.len() as u64 - rdr.position();
                        log::debug!(
                            "payload length {} - reader_position {} = {remaining_bytes}",
                            payload.len(),
                            rdr.position()
                        );
                        if length < remaining_bytes {
                            let read = std::io::copy(&mut rdr.by_ref().take(length), &mut output)?;
                            log::debug!(
                                "expected to read {length} bytes, actually read {read} bytes"
                            );
                        } else {
                            let read = std::io::copy(&mut rdr.take(remaining_bytes), &mut output)?;
                            len.replace(length - remaining_bytes);
                            log::debug!("expected to read {remaining_bytes} bytes, actually read {read} bytes");
                            // this payload is exhausted
                            break;
                        }
                    }
                }
            }
            if Cursor::new(&payload[(payload.len() - 8)..(payload.len() - 4)])
                .read_u32::<LittleEndian>()?
                == MessageSubcommand::Done as u32
            {
                break;
            }
        }
        Ok(())
    }

    pub(crate) fn push_file<R: std::io::Read>(
        &mut self,
        local_id: u32,
        remote_id: u32,
        mut reader: R,
    ) -> std::result::Result<(), RustADBError> {
        let mut buffer = [0; BUFFER_SIZE];
        let amount_read = reader.read(&mut buffer)?;
        let subcommand_data = MessageSubcommand::Data.with_arg(amount_read as u32);

        let mut serialized_message =
            bincode::serialize(&subcommand_data).map_err(|_e| RustADBError::ConversionError)?;
        serialized_message.append(&mut buffer[..amount_read].to_vec());

        let message = ADBTransportMessage::new(
            MessageCommand::Write,
            local_id,
            remote_id,
            serialized_message,
        );

        self.send_and_expect_okay(message)?;

        loop {
            let mut buffer = [0; BUFFER_SIZE];

            match reader.read(&mut buffer) {
                Ok(0) => {
                    // Currently file mtime is not forwarded
                    let subcommand_data = MessageSubcommand::Done.with_arg(0);

                    let serialized_message = bincode::serialize(&subcommand_data)
                        .map_err(|_e| RustADBError::ConversionError)?;

                    let message = ADBTransportMessage::new(
                        MessageCommand::Write,
                        local_id,
                        remote_id,
                        serialized_message,
                    );

                    self.send_and_expect_okay(message)?;

                    // Command should end with a Write => Okay
                    let received = self.transport.read_message()?;
                    match received.header().command() {
                        MessageCommand::Write => return Ok(()),
                        c => {
                            return Err(RustADBError::ADBRequestFailed(format!(
                                "Wrong command received {}",
                                c
                            )))
                        }
                    }
                }
                Ok(size) => {
                    let subcommand_data = MessageSubcommand::Data.with_arg(size as u32);

                    let mut serialized_message = bincode::serialize(&subcommand_data)
                        .map_err(|_e| RustADBError::ConversionError)?;
                    serialized_message.append(&mut buffer[..size].to_vec());

                    let message = ADBTransportMessage::new(
                        MessageCommand::Write,
                        local_id,
                        remote_id,
                        serialized_message,
                    );

                    self.send_and_expect_okay(message)?;
                }
                Err(e) => {
                    return Err(RustADBError::IOError(e));
                }
            }
        }
    }

    pub(crate) fn begin_synchronization(&mut self) -> Result<(u32, u32)> {
        let sync_directive = "sync:\0";

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
        Ok((local_id, remote_id))
    }

    pub(crate) fn stat_with_explicit_ids(
        &mut self,
        remote_path: &str,
        local_id: u32,
        remote_id: u32,
    ) -> Result<AdbStatResponse> {
        let stat_buffer = MessageSubcommand::Stat.with_arg(remote_path.len() as u32);
        let message = ADBTransportMessage::new(
            MessageCommand::Write,
            local_id,
            remote_id,
            bincode::serialize(&stat_buffer).map_err(|_e| RustADBError::ConversionError)?,
        );
        self.send_and_expect_okay(message)?;
        self.send_and_expect_okay(ADBTransportMessage::new(
            MessageCommand::Write,
            local_id,
            remote_id,
            remote_path.into(),
        ))?;
        let response = self.transport.read_message()?;
        // Skip first 4 bytes as this is the literal "STAT".
        // Interesting part starts right after
        bincode::deserialize(&response.into_payload()[4..])
            .map_err(|_e| RustADBError::ConversionError)
    }

    pub(crate) fn end_transaction(&mut self, local_id: u32, remote_id: u32) -> Result<()> {
        let quit_buffer = MessageSubcommand::Quit.with_arg(0u32);
        self.send_and_expect_okay(ADBTransportMessage::new(
            MessageCommand::Write,
            local_id,
            remote_id,
            bincode::serialize(&quit_buffer).map_err(|_e| RustADBError::ConversionError)?,
        ))?;
        let _discard_close = self.transport.read_message()?;
        Ok(())
    }
}
