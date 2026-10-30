use std::{collections::VecDeque, io::Read, sync::mpsc::Sender, time::SystemTime};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{
    AdbStatResponse, BinaryDecodable, Result, RustADBError,
    message_devices::{
        adb_message_transport::ADBMessageTransport,
        adb_transport_message::ADBTransportMessage,
        message_commands::{MessageCommand, MessageSubcommand},
        message_dispatcher::{DispatcherCommand, StreamReceiver, connection_lost_error},
        utils::BinaryEncodable,
    },
};

/// Maximum size of the data of a sync `DATA` chunk.
const SYNC_DATA_MAX_SIZE: usize = 64 * 1024;

/// Size of a sync chunk header (identifier and length).
const SYNC_HEADER_SIZE: usize = 8;

/// Represent a session between an `ADBDevice` and remote `adbd`, closed when dropped.
#[derive(Debug)]
pub struct ADBSession<T: ADBMessageTransport> {
    transport: T,
    /// Messages received for this session.
    receiver: StreamReceiver,
    /// Commands sent to the reader thread.
    dispatcher_commands: Sender<DispatcherCommand>,
    local_id: u32,
    remote_id: u32,
    max_payload_size: usize,
    /// The session has been closed, by either side.
    closed: bool,
}

impl<T: ADBMessageTransport> ADBSession<T> {
    /// Create a session not opened yet: [`ADBSession::opened`] must be called once the device accepted it.
    pub(crate) const fn new(
        transport: T,
        local_id: u32,
        receiver: StreamReceiver,
        dispatcher_commands: Sender<DispatcherCommand>,
        max_payload_size: usize,
    ) -> Self {
        Self {
            transport,
            receiver,
            dispatcher_commands,
            local_id,
            remote_id: 0,
            max_payload_size,
            // Nothing to close until the device accepts the session
            closed: true,
        }
    }

    /// Mark the session as opened by the device, under `remote_id`.
    pub(crate) const fn opened(&mut self, remote_id: u32) {
        self.remote_id = remote_id;
        self.closed = false;
    }

    pub const fn local_id(&self) -> u32 {
        self.local_id
    }

    pub const fn remote_id(&self) -> u32 {
        self.remote_id
    }

    /// Maximum payload size of a message sent to the device.
    pub const fn max_payload_size(&self) -> usize {
        self.max_payload_size
    }

    /// Send `message` to the device.
    pub(crate) fn send(&mut self, message: ADBTransportMessage) -> Result<()> {
        self.transport.write_message(message)
    }

    /// Send an `OKAY` message, telling the device we are ready to receive more data.
    pub(crate) fn send_okay(&mut self) -> Result<()> {
        self.send(ADBTransportMessage::try_new(
            MessageCommand::Okay,
            self.local_id,
            self.remote_id,
            &[],
        )?)
    }

    /// Receive the next message of this session.
    pub(crate) fn recv(&mut self) -> Result<ADBTransportMessage> {
        let message = self
            .receiver
            .recv()
            .map_err(|_| connection_lost_error("message reader stopped"))??;
        if message.header().command() == MessageCommand::Clse {
            self.closed = true;
        }
        Ok(message)
    }

    /// Receive a message and acknowledge it by replying with an `OKAY` command if it carries data.
    pub(crate) fn recv_and_reply_okay(&mut self) -> Result<ADBTransportMessage> {
        let message = self.recv()?;
        if message.header().command() == MessageCommand::Write {
            self.send_okay()?;
        }
        Ok(message)
    }

    /// Expect a message with an `OKAY` command after sending a message.
    pub(crate) fn send_and_expect_okay(
        &mut self,
        message: ADBTransportMessage,
    ) -> Result<ADBTransportMessage> {
        self.send(message)?;

        let message = self.recv()?;
        message.assert_command(MessageCommand::Okay)?;
        Ok(message)
    }

    /// Close the session from our side, unless it is already closed.
    pub(crate) fn close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }

        self.closed = true;
        self.send(ADBTransportMessage::try_new(
            MessageCommand::Clse,
            self.local_id,
            self.remote_id,
            &[],
        )?)
    }

    /// End a sync transaction, and wait for the device to close the session.
    pub(crate) fn end_transaction(&mut self) -> Result<()> {
        let quit_buffer = MessageSubcommand::Quit.with_arg(0u32);
        self.send_and_expect_okay(ADBTransportMessage::try_new(
            MessageCommand::Write,
            self.local_id,
            self.remote_id,
            &quit_buffer.encode(),
        )?)?;

        self.recv_until_close()?;
        Ok(())
    }

    /// Receive the data sent by the device until it closes the session, and return it.
    ///
    /// `OKAY` messages only tell that the device is ready for more data, they are skipped. `adbd` sends one each
    /// time it forwards part of its queue to the service, so a single message can be acknowledged several times
    /// when the service reads slowly.
    pub(crate) fn recv_until_close(&mut self) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        loop {
            let message = self.recv_and_reply_okay()?;
            match message.header().command() {
                MessageCommand::Okay => {}
                MessageCommand::Write => output.extend_from_slice(message.payload()),
                MessageCommand::Clse => return Ok(output),
                command => {
                    return Err(RustADBError::ADBRequestFailed(format!(
                        "Wrong command received {command}"
                    )));
                }
            }
        }
    }

    /// Receive a file sent by the device as sync chunks, and write its content to `output`.
    ///
    /// `DATA` chunks carry the content, `DONE` ends the transfer and `FAIL` carries an error. Chunks are not
    /// aligned on messages: a message can hold several chunks, and a chunk (or its header) can span several messages.
    pub(crate) fn recv_file<W: std::io::Write>(&mut self, mut output: W) -> Result<()> {
        // Data received from the device, not parsed yet
        let mut pending = VecDeque::new();
        loop {
            self.recv_sync_data(&mut pending, SYNC_HEADER_SIZE)?;
            let id = pending.read_u32::<LittleEndian>()?;
            let length = pending.read_u32::<LittleEndian>()? as usize;

            match MessageSubcommand::try_from(id) {
                Ok(MessageSubcommand::Data) => {
                    // Same limit as adb, the device never sends bigger chunks
                    if length > SYNC_DATA_MAX_SIZE {
                        return Err(RustADBError::ADBRequestFailed(format!(
                            "sync DATA chunk of {length} bytes is over the {SYNC_DATA_MAX_SIZE} bytes limit"
                        )));
                    }

                    let mut remaining = length as u64;
                    while remaining > 0 {
                        self.recv_sync_data(&mut pending, 1)?;
                        remaining -=
                            std::io::copy(&mut (&mut pending).take(remaining), &mut output)?;
                    }
                }
                Ok(MessageSubcommand::Done) => return Ok(()),
                Ok(MessageSubcommand::Fail) => {
                    self.recv_sync_data(&mut pending, length)?;
                    let reason: Vec<u8> = pending.drain(..length).collect();
                    return Err(RustADBError::ADBRequestFailed(
                        String::from_utf8_lossy(&reason).into_owned(),
                    ));
                }
                _ => {
                    return Err(RustADBError::ADBRequestFailed(format!(
                        "unexpected sync chunk id {id:#010x} while receiving a file"
                    )));
                }
            }
        }
    }

    /// Receive data from the device until `pending` holds at least `size` bytes.
    fn recv_sync_data(&mut self, pending: &mut VecDeque<u8>, size: usize) -> Result<()> {
        while pending.len() < size {
            let message = self.recv_and_reply_okay()?;
            match message.header().command() {
                MessageCommand::Okay => {}
                MessageCommand::Write => pending.extend(message.payload()),
                MessageCommand::Clse => {
                    return Err(RustADBError::ADBRequestFailed(
                        "device closed the session during a sync transfer".into(),
                    ));
                }
                command => {
                    return Err(RustADBError::ADBRequestFailed(format!(
                        "Wrong command received {command}"
                    )));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn push_file<R: std::io::Read>(&mut self, mut reader: R) -> Result<()> {
        let mut buffer = vec![0; SYNC_DATA_MAX_SIZE].into_boxed_slice();
        let mut eof = false;

        while !eof {
            // Fill the message with as many sync `DATA` chunks as it can hold
            let mut payload = Vec::with_capacity(self.max_payload_size);
            while payload.len() + SYNC_HEADER_SIZE < self.max_payload_size {
                let chunk_size = (self.max_payload_size - payload.len() - SYNC_HEADER_SIZE)
                    .min(SYNC_DATA_MAX_SIZE);
                let size = reader.read(&mut buffer[..chunk_size])?;
                if size == 0 {
                    eof = true;
                    break;
                }

                payload.append(
                    &mut MessageSubcommand::Data
                        .with_arg(u32::try_from(size)?)
                        .encode(),
                );
                payload.extend_from_slice(&buffer[..size]);
            }

            if !payload.is_empty() {
                self.send_and_expect_okay(ADBTransportMessage::try_new(
                    MessageCommand::Write,
                    self.local_id(),
                    self.remote_id(),
                    &payload,
                )?)?;
            }
        }

        // `DONE` carries the file modification time: a stream has none, the file is dated from now
        let Ok(last_modified) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) else {
            return Err(RustADBError::ADBRequestFailed(
                "SystemTime before unix epoch".into(),
            ));
        };
        let subcommand_data =
            MessageSubcommand::Done.with_arg(u32::try_from(last_modified.as_secs())?);

        let message = ADBTransportMessage::try_new(
            MessageCommand::Write,
            self.local_id(),
            self.remote_id(),
            &subcommand_data.encode(),
        )?;

        self.send_and_expect_okay(message)?;

        // Command should end with a Write => Okay
        let received = self.recv_and_reply_okay()?;
        match received.header().command() {
            MessageCommand::Write => Ok(()),
            c => Err(RustADBError::ADBRequestFailed(format!(
                "Wrong command received {c}"
            ))),
        }
    }

    pub(crate) fn stat_with_explicit_ids(&mut self, remote_path: &str) -> Result<AdbStatResponse> {
        let stat_buffer = MessageSubcommand::Stat.with_arg(u32::try_from(remote_path.len())?);
        let message = ADBTransportMessage::try_new(
            MessageCommand::Write,
            self.local_id(),
            self.remote_id(),
            &stat_buffer.encode(),
        )?;
        self.send_and_expect_okay(message)?;
        self.send_and_expect_okay(ADBTransportMessage::try_new(
            MessageCommand::Write,
            self.local_id(),
            self.remote_id(),
            remote_path.as_bytes(),
        )?)?;

        let response = self.recv_and_reply_okay()?;
        // Skip first 4 bytes as this is the literal "STAT".
        // Interesting part starts right after

        AdbStatResponse::decode(&response.into_payload()[4..])
    }
}

impl<T: ADBMessageTransport> Drop for ADBSession<T> {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.close();
        let _ = self
            .dispatcher_commands
            .send(DispatcherCommand::Unregister(self.local_id));
    }
}
