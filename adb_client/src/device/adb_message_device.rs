use byteorder::{LittleEndian, ReadBytesExt};
use rand::Rng;
use std::io::{Cursor, Read, Seek};

use crate::{ADBMessageTransport, AdbStatResponse, Result, RustADBError, constants::BUFFER_SIZE};

use super::{ADBTransportMessage, MessageCommand, models::MessageSubcommand};

/// Generic structure representing an ADB device reachable over an [`ADBMessageTransport`].
/// Structure is totally agnostic over which transport is truly used.
#[derive(Debug)]
pub struct ADBMessageDevice<T: ADBMessageTransport> {
    transport: T,
    local_id: Option<u32>,
    remote_id: Option<u32>,
}

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    /// Instantiate a new [`ADBMessageTransport`]
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            local_id: None,
            remote_id: None,
        }
    }

    pub(crate) fn get_transport(&mut self) -> &T {
        &self.transport
    }

    pub(crate) fn get_transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Receive a message and acknowledge it by replying with an `OKAY` command
    pub(crate) fn recv_and_reply_okay(&mut self) -> Result<ADBTransportMessage> {
        let message = self.transport.read_message()?;
        self.transport.write_message(ADBTransportMessage::new(
            MessageCommand::Okay,
            self.get_local_id()?,
            self.get_remote_id()?,
            &[],
        ))?;
        Ok(message)
    }

    /// Expect a message with an `OKAY` command after sending a message.
    pub(crate) fn send_and_expect_okay(
        &mut self,
        message: ADBTransportMessage,
    ) -> Result<ADBTransportMessage> {
        self.transport.write_message(message)?;

        self.transport.read_message().and_then(|message| {
            message.assert_command(MessageCommand::Okay)?;
            Ok(message)
        })
    }

    pub(crate) fn recv_file<W: std::io::Write>(
        &mut self,
        mut output: W,
    ) -> std::result::Result<(), RustADBError> {
        let mut len: Option<u64> = None;
        loop {
            let payload = self.recv_and_reply_okay()?.into_payload();
            let mut rdr = Cursor::new(&payload);
            while rdr.position() != payload.len() as u64 {
                match len.take() {
                    Some(0) | None => {
                        rdr.seek_relative(4)?;
                        len.replace(rdr.read_u32::<LittleEndian>()? as u64);
                    }
                    Some(length) => {
                        let remaining_bytes = payload.len() as u64 - rdr.position();
                        if length < remaining_bytes {
                            std::io::copy(&mut rdr.by_ref().take(length), &mut output)?;
                        } else {
                            std::io::copy(&mut rdr.take(remaining_bytes), &mut output)?;
                            len.replace(length - remaining_bytes);
                            // this payload is now exhausted
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
            &serialized_message,
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
                        &serialized_message,
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
                            )));
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
                        &serialized_message,
                    );

                    self.send_and_expect_okay(message)?;
                }
                Err(e) => {
                    return Err(RustADBError::IOError(e));
                }
            }
        }
    }

    pub(crate) fn begin_synchronization(&mut self) -> Result<()> {
        self.open_session(b"sync:\0")?;
        Ok(())
    }

    pub(crate) fn stat_with_explicit_ids(&mut self, remote_path: &str) -> Result<AdbStatResponse> {
        let stat_buffer = MessageSubcommand::Stat.with_arg(remote_path.len() as u32);
        let message = ADBTransportMessage::new(
            MessageCommand::Write,
            self.get_local_id()?,
            self.get_remote_id()?,
            &bincode::serialize(&stat_buffer).map_err(|_e| RustADBError::ConversionError)?,
        );
        self.send_and_expect_okay(message)?;
        self.send_and_expect_okay(ADBTransportMessage::new(
            MessageCommand::Write,
            self.get_local_id()?,
            self.get_remote_id()?,
            remote_path.as_bytes(),
        ))?;
        let response = self.transport.read_message()?;
        // Skip first 4 bytes as this is the literal "STAT".
        // Interesting part starts right after
        bincode::deserialize(&response.into_payload()[4..])
            .map_err(|_e| RustADBError::ConversionError)
    }

    pub(crate) fn end_transaction(&mut self) -> Result<()> {
        let quit_buffer = MessageSubcommand::Quit.with_arg(0u32);
        self.send_and_expect_okay(ADBTransportMessage::new(
            MessageCommand::Write,
            self.get_local_id()?,
            self.get_remote_id()?,
            &bincode::serialize(&quit_buffer).map_err(|_e| RustADBError::ConversionError)?,
        ))?;
        let _discard_close = self.transport.read_message()?;
        Ok(())
    }

    pub(crate) fn open_session(&mut self, data: &[u8]) -> Result<ADBTransportMessage> {
        let mut rng = rand::rng();

        let message = ADBTransportMessage::new(
            MessageCommand::Open,
            rng.random(), // Our 'local-id'
            0,
            data,
        );
        self.get_transport_mut().write_message(message)?;

        let response = self.get_transport_mut().read_message()?;

        self.local_id = Some(response.header().arg1());
        self.remote_id = Some(response.header().arg0());

        Ok(response)
    }

    pub(crate) fn get_local_id(&self) -> Result<u32> {
        self.local_id.ok_or(RustADBError::ADBRequestFailed(
            "connection not opened, no local_id".into(),
        ))
    }

    pub(crate) fn get_remote_id(&self) -> Result<u32> {
        self.remote_id.ok_or(RustADBError::ADBRequestFailed(
            "connection not opened, no remote_id".into(),
        ))
    }
}
