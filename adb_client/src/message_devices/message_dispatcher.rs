use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicU32, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread::JoinHandle,
    time::Duration,
};

use rand::RngExt;

use crate::{
    Result, RustADBError,
    message_devices::{
        adb_message_transport::ADBMessageTransport, adb_session::ADBSession,
        adb_transport_message::ADBTransportMessage, commands::accept_reverse_connection,
        message_commands::MessageCommand,
    },
};

/// How long the reader thread waits for a message before handling pending commands.
const READER_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Messages received for a stream, or the error that lost the connection.
pub type StreamReceiver = Receiver<Result<ADBTransportMessage>>;
type StreamSender = Sender<Result<ADBTransportMessage>>;

/// Command sent to the reader thread.
#[derive(Debug)]
pub enum DispatcherCommand {
    /// Send messages of stream with this local id to this channel.
    Register(u32, StreamSender),
    /// Stop sending messages of stream with this local id. Later ones are handled as messages for an unknown
    /// stream.
    Unregister(u32),
    /// Stop the reader thread.
    Stop,
}

/// Route messages received on a transport shared by several streams to the stream they belong to.
///
/// A dedicated thread reads every message received and sends it to the channel of its stream. It only knows streams
/// through the [`DispatcherCommand`]s it receives. Messages for unknown streams (e.g. a stream we already closed)
/// are answered like `adb` does, and never reach another stream. Streams opened by the device are only accepted
/// for reverse forwards set up in [`DispatcherHandle::reverses`].
///
/// The reader thread is stopped when dropped.
#[derive(Debug)]
pub struct MessageDispatcher {
    handle: DispatcherHandle,
    reader: Option<JoinHandle<()>>,
}

impl MessageDispatcher {
    /// Start reading messages of `transport`, on which messages carry at most `max_payload_size` bytes.
    pub(crate) fn start<T: ADBMessageTransport>(
        transport: T,
        max_payload_size: usize,
    ) -> Result<Self> {
        let (commands, commands_receiver) = mpsc::channel();
        let handle = DispatcherHandle {
            commands,
            // Start at a random id, to not match messages of a previous connection
            next_local_id: Arc::new(AtomicU32::new(rand::rng().random_range(1..u32::MAX))),
            reverses: Arc::default(),
        };

        let reader_handle = handle.clone();
        let reader = std::thread::Builder::new()
            .name("adb-message-reader".into())
            .spawn(move || {
                MessageReader {
                    transport,
                    commands: commands_receiver,
                    handle: reader_handle,
                    max_payload_size,
                    streams: HashMap::new(),
                }
                .run();
            })?;

        Ok(Self {
            handle,
            reader: Some(reader),
        })
    }

    /// Handle to talk to the reader thread, usable from other threads.
    pub(crate) const fn handle(&self) -> &DispatcherHandle {
        &self.handle
    }

    /// Stop the reader thread, and wait for it to release the transport.
    pub(crate) fn stop(&mut self) {
        let _ = self.handle.send(DispatcherCommand::Stop);
        if let Some(reader) = self.reader.take()
            && reader.join().is_err()
        {
            log::error!("message reader thread panicked");
        }
    }
}

impl Drop for MessageDispatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Cloneable handle to the reader thread of a [`MessageDispatcher`].
#[derive(Clone, Debug)]
pub struct DispatcherHandle {
    commands: Sender<DispatcherCommand>,
    /// Next local id to give to a stream.
    next_local_id: Arc<AtomicU32>,
    /// Host side of each reverse forward, by device side. Streams opened by the device to a host side are accepted.
    reverses: Arc<Mutex<HashMap<String, String>>>,
}

impl DispatcherHandle {
    /// Register a new stream, and return its local id and the channel receiving its messages.
    ///
    /// Messages received for this stream are sent from now on, so register it before sending its `OPEN`.
    pub(crate) fn register(&self) -> Result<(u32, StreamReceiver)> {
        let local_id = self.next_local_id();
        let (sender, receiver) = mpsc::channel();
        self.send(DispatcherCommand::Register(local_id, sender))?;
        Ok((local_id, receiver))
    }

    /// Send `command` to the reader thread.
    pub(crate) fn send(&self, command: DispatcherCommand) -> Result<()> {
        self.commands
            .send(command)
            .map_err(|_| connection_lost_error("message reader stopped"))
    }

    /// Channel sending commands to the reader thread.
    pub(crate) fn commands(&self) -> Sender<DispatcherCommand> {
        self.commands.clone()
    }

    /// Host side of each reverse forward, by device side.
    pub(crate) fn reverses(&self) -> Result<MutexGuard<'_, HashMap<String, String>>> {
        self.reverses.lock().map_err(|_| RustADBError::PoisonError)
    }

    fn next_local_id(&self) -> u32 {
        match self.next_local_id.fetch_add(1, Ordering::Relaxed) {
            // 0 is not a valid stream id
            0 => self.next_local_id.fetch_add(1, Ordering::Relaxed),
            local_id => local_id,
        }
    }
}

/// Error returned to streams once the connection has been lost because of `reason`.
pub fn connection_lost_error(reason: &str) -> RustADBError {
    RustADBError::IOError(std::io::Error::new(
        std::io::ErrorKind::ConnectionAborted,
        format!("connection lost: {reason}"),
    ))
}

/// State of the reader thread.
struct MessageReader<T: ADBMessageTransport> {
    transport: T,
    commands: Receiver<DispatcherCommand>,
    /// Used to create the sessions of streams opened by the device.
    handle: DispatcherHandle,
    max_payload_size: usize,
    /// Channel of each open stream, by local id.
    streams: HashMap<u32, StreamSender>,
}

impl<T: ADBMessageTransport> MessageReader<T> {
    fn run(mut self) {
        let reason = match self.read_messages() {
            Ok(()) => "connection closed".to_string(),
            Err(e) => e.to_string(),
        };
        log::debug!("message reader stopped: {reason}");

        // Streams may have been registered since the last commands were handled
        let _ = self.handle_commands();
        for sender in self.streams.values() {
            let _ = sender.send(Err(connection_lost_error(&reason)));
        }
    }

    /// Read messages and dispatch them, until asked to stop or the transport fails.
    fn read_messages(&mut self) -> Result<()> {
        loop {
            let message = match self
                .transport
                .read_message_with_timeout(READER_POLL_INTERVAL)
            {
                Ok(message) => Some(message),
                Err(e) if e.is_timeout() => None,
                Err(e) => return Err(e),
            };

            // Handled after reading: a stream is registered before its `OPEN` is sent, so before its answer is read
            if !self.handle_commands() {
                return Ok(());
            }

            if let Some(message) = message
                && let Some(reply) = self.dispatch(message)?
            {
                // Best effort, the stream is unknown to us anyway
                if let Err(e) = self.transport.write_message(reply) {
                    log::debug!("cannot answer a message of an unknown stream: {e}");
                }
            }
        }
    }

    /// Handle every pending command. Return `false` if the reader must stop.
    fn handle_commands(&mut self) -> bool {
        while let Ok(command) = self.commands.try_recv() {
            match command {
                DispatcherCommand::Register(local_id, sender) => {
                    self.streams.insert(local_id, sender);
                }
                DispatcherCommand::Unregister(local_id) => {
                    self.streams.remove(&local_id);
                }
                DispatcherCommand::Stop => return false,
            }
        }
        true
    }

    /// Send `message` to the stream it belongs to. Return the reply to send if it belongs to no known stream.
    fn dispatch(&mut self, message: ADBTransportMessage) -> Result<Option<ADBTransportMessage>> {
        let header = message.header();
        let (command, remote_id, local_id) = (header.command(), header.arg0(), header.arg1());

        match command {
            MessageCommand::Okay | MessageCommand::Write | MessageCommand::Clse => {
                // Sending fails if the session has been dropped, before unregistering
                let sent = self
                    .streams
                    .get(&local_id)
                    .is_some_and(|sender| sender.send(Ok(message)).is_ok());
                if sent {
                    return Ok(None);
                }

                self.streams.remove(&local_id);
                log::debug!("discarding {command} received for unknown stream {local_id}");
                if command == MessageCommand::Clse {
                    return Ok(None);
                }
            }
            MessageCommand::Open => {
                // Destination is NUL-terminated
                let destination = String::from_utf8_lossy(message.payload());
                let destination = destination.trim_end_matches('\0');

                // Only destinations of reverse forwards are served, like adb does
                let reversed = self
                    .handle
                    .reverses()
                    .is_ok_and(|reverses| reverses.values().any(|local| local == destination));
                if reversed {
                    self.accept(remote_id, destination.to_string());
                    return Ok(None);
                }
                log::debug!("refusing stream opened by device to {destination}");
            }
            command => {
                return Err(RustADBError::ADBRequestFailed(format!(
                    "unexpected {command} message received on an established connection"
                )));
            }
        }

        // Tell the device that this stream does not exist
        Ok(Some(ADBTransportMessage::try_new(
            MessageCommand::Clse,
            0,
            remote_id,
            &[],
        )?))
    }

    /// Accept a stream opened by the device under `remote_id` to `destination`.
    fn accept(&mut self, remote_id: u32, destination: String) {
        // Registered right away: device messages for this stream follow our `OKAY`
        let local_id = self.handle.next_local_id();
        let (sender, receiver) = mpsc::channel();
        self.streams.insert(local_id, sender);

        let session = ADBSession::new(
            self.transport.clone(),
            local_id,
            receiver,
            self.handle.commands(),
            self.max_payload_size,
        )
        .incoming(remote_id);
        accept_reverse_connection(session, destination);
    }
}
