use std::{
    collections::VecDeque,
    io::Read,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    time::SystemTime,
};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{
    AdbStatResponse, BinaryDecodable, Result, RustADBError,
    message_devices::{
        adb_message_transport::ADBMessageTransport,
        adb_transport_message::ADBTransportMessage,
        message_commands::{MessageCommand, MessageSubcommand},
        message_dispatcher::{
            DispatcherCommand, DispatcherHandle, StreamReceiver, connection_lost_error,
        },
        utils::BinaryEncodable,
    },
    models::ADBLocalCommand,
};

/// Maximum size of the data of a sync `DATA` chunk.
const SYNC_DATA_MAX_SIZE: usize = 64 * 1024;

/// Size of a sync chunk header (identifier and length).
const SYNC_HEADER_SIZE: usize = 8;

/// Represent a session between an `ADBDevice` and remote `adbd`, closed when dropped.
#[derive(Debug)]
pub struct ADBSession<T: ADBMessageTransport> {
    link: SessionLink<T>,
    /// Messages received for this session.
    receiver: StreamReceiver,
    max_payload_size: usize,
    /// `false` for a stream opened by the device and not accepted yet: closing it refuses it.
    accepted: bool,
}

impl<T: ADBMessageTransport> ADBSession<T> {
    /// Create a session not opened yet: [`ADBSession::opened`] must be called once the device accepted it.
    pub(crate) fn new(
        transport: T,
        local_id: u32,
        receiver: StreamReceiver,
        dispatcher_commands: Sender<DispatcherCommand>,
        max_payload_size: usize,
    ) -> Self {
        Self {
            link: SessionLink {
                transport,
                dispatcher_commands,
                local_id,
                remote_id: 0,
                // Nothing to close until the device accepts the session
                closed: Arc::new(AtomicBool::new(true)),
            },
            receiver,
            max_payload_size,
            accepted: true,
        }
    }

    /// Mark the session as opened by the device, under `remote_id`.
    pub(crate) fn opened(&mut self, remote_id: u32) {
        self.link.remote_id = remote_id;
        self.link.closed.store(false, Ordering::SeqCst);
    }

    /// Turn this session into the one of a stream opened by the device under `remote_id`, not accepted yet:
    /// [`ADBSession::accept`] accepts it, closing or dropping it before refuses it.
    pub(crate) fn incoming(mut self, remote_id: u32) -> Self {
        self.opened(remote_id);
        self.accepted = false;
        self
    }

    /// Accept a stream opened by the device, see [`ADBSession::incoming`].
    pub(crate) fn accept(&mut self) -> Result<()> {
        self.send_okay()?;
        self.accepted = true;
        Ok(())
    }

    /// Return a sending half of this session, to send data from another thread than the one receiving.
    pub(crate) fn writer(&self) -> ADBSessionWriter<T> {
        ADBSessionWriter {
            link: self.link.clone(),
        }
    }

    pub const fn local_id(&self) -> u32 {
        self.link.local_id
    }

    pub const fn remote_id(&self) -> u32 {
        self.link.remote_id
    }

    /// Maximum payload size of a message sent to the device.
    pub const fn max_payload_size(&self) -> usize {
        self.max_payload_size
    }

    /// Send `message` to the device.
    pub(crate) fn send(&mut self, message: ADBTransportMessage) -> Result<()> {
        self.link.transport.write_message(message)
    }

    /// Send an `OKAY` message, telling the device we are ready to receive more data.
    pub(crate) fn send_okay(&mut self) -> Result<()> {
        self.link.send(MessageCommand::Okay, &[])
    }

    /// Receive the next message of this session.
    pub(crate) fn recv(&self) -> Result<ADBTransportMessage> {
        let message = self
            .receiver
            .recv()
            .map_err(|_| connection_lost_error("message reader stopped"))??;
        if message.header().command() == MessageCommand::Clse {
            self.link.closed.store(true, Ordering::SeqCst);
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

    /// Send `payload` in a `WRTE` message, and expect the device to acknowledge it with an `OKAY`.
    pub(crate) fn send_write_and_expect_okay(&mut self, payload: &[u8]) -> Result<()> {
        self.link.send(MessageCommand::Write, payload)?;
        self.recv()?.assert_command(MessageCommand::Okay)
    }

    /// Close the session from our side, unless it is already closed.
    pub(crate) fn close(&mut self) -> Result<()> {
        if self.accepted {
            self.link.close()
        } else {
            self.link.refuse()
        }
    }

    /// End a sync transaction, and wait for the device to close the session.
    pub(crate) fn end_transaction(&mut self) -> Result<()> {
        let quit_buffer = MessageSubcommand::Quit.with_arg(0u32);
        self.send_write_and_expect_okay(&quit_buffer.encode())?;

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
                self.send_write_and_expect_okay(&payload)?;
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

        self.send_write_and_expect_okay(&subcommand_data.encode())?;

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
        self.send_write_and_expect_okay(&stat_buffer.encode())?;
        self.send_write_and_expect_okay(remote_path.as_bytes())?;

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
        self.link.unregister();
    }
}

/// What both halves of a session need to send its messages.
#[derive(Clone, Debug)]
struct SessionLink<T: ADBMessageTransport> {
    transport: T,
    dispatcher_commands: Sender<DispatcherCommand>,
    local_id: u32,
    remote_id: u32,
    /// The session has been closed, by either side. Shared by both halves.
    closed: Arc<AtomicBool>,
}

impl<T: ADBMessageTransport> SessionLink<T> {
    /// Send a `command` message of this session.
    fn send(&mut self, command: MessageCommand, payload: &[u8]) -> Result<()> {
        self.transport.write_message(ADBTransportMessage::try_new(
            command,
            self.local_id,
            self.remote_id,
            payload,
        )?)
    }

    /// Close the session, unless it is already closed.
    fn close(&mut self) -> Result<()> {
        self.send_close(self.local_id)
    }

    /// Refuse a stream opened by the device, unless it is already closed. Like adb, its `CLSE` has local id 0: the
    /// stream has no peer on the device yet.
    fn refuse(&mut self) -> Result<()> {
        self.send_close(0)
    }

    /// Send a `CLSE` message with `local_id`, unless the session is already closed.
    fn send_close(&mut self, local_id: u32) -> Result<()> {
        if self.closed.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        self.transport.write_message(ADBTransportMessage::try_new(
            MessageCommand::Clse,
            local_id,
            self.remote_id,
            &[],
        )?)
    }

    /// Stop receiving messages of this session: a thread waiting for one gets an error.
    fn unregister(&self) {
        // Best effort, the reader thread may be gone
        let _ = self
            .dispatcher_commands
            .send(DispatcherCommand::Unregister(self.local_id));
    }
}

/// Sending half of an [`ADBSession`], used to send data while another thread receives the session messages.
#[derive(Debug)]
pub struct ADBSessionWriter<T: ADBMessageTransport> {
    link: SessionLink<T>,
}

impl<T: ADBMessageTransport> ADBSessionWriter<T> {
    /// Send `data` to the device in a single `WRTE` message. It must fit in the session maximum payload size.
    pub(crate) fn write(&mut self, data: &[u8]) -> Result<()> {
        if self.link.closed.load(Ordering::SeqCst) {
            return Err(
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "session closed").into(),
            );
        }

        self.link.send(MessageCommand::Write, data)
    }

    /// Close the session from our side, unless it is already closed.
    ///
    /// The session stops receiving messages: a thread waiting for one gets an error.
    pub(crate) fn close(&mut self) -> Result<()> {
        let result = self.link.close();
        // The device may never acknowledge our `CLSE`, do not wait for it
        self.link.unregister();
        result
    }
}

/// Open sessions on a device, from any thread.
#[derive(Clone, Debug)]
pub struct ADBSessionOpener<T: ADBMessageTransport> {
    transport: T,
    dispatcher: DispatcherHandle,
    max_payload_size: usize,
}

impl<T: ADBMessageTransport> ADBSessionOpener<T> {
    pub(crate) const fn new(
        transport: T,
        dispatcher: DispatcherHandle,
        max_payload_size: usize,
    ) -> Self {
        Self {
            transport,
            dispatcher,
            max_payload_size,
        }
    }

    /// Open a new ADB session with the given local command.
    pub(crate) fn open(&self, cmd: &ADBLocalCommand) -> Result<ADBSession<T>> {
        // Registered before sending `OPEN`, so the answer is routed to it
        let (local_id, receiver) = self.dispatcher.register()?;
        let mut session = ADBSession::new(
            self.transport.clone(),
            local_id,
            receiver,
            self.dispatcher.commands(),
            self.max_payload_size,
        );

        // adbd used to expect a null-terminated string.
        // keep doing so to maintain compatibility with older versions.
        // https://cs.android.com/android/platform/superproject/+/android-latest-release:packages/modules/adb/sockets.cpp;l=560?q=sockets.cpp
        let mut destination = cmd.to_string().into_bytes();
        if !destination.ends_with(&[0]) {
            destination.push(0);
        }

        session.send(ADBTransportMessage::try_new(
            MessageCommand::Open,
            local_id,
            0,
            &destination,
        )?)?;

        let response = session.recv()?;
        match response.header().command() {
            MessageCommand::Okay => {
                session.opened(response.header().arg0());
                Ok(session)
            }
            // The device could not open the service
            MessageCommand::Clse => Err(RustADBError::ADBRequestFailed(format!(
                "Open session failed: device closed the stream to {cmd}"
            ))),
            command => Err(RustADBError::ADBRequestFailed(format!(
                "Open session failed: got {command} in response instead of OKAY"
            ))),
        }
    }
}
