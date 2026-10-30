use std::{
    collections::HashMap,
    sync::{
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
        adb_message_transport::ADBMessageTransport, adb_transport_message::ADBTransportMessage,
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
/// are answered like `adb` does, and never reach another stream.
///
/// The reader thread is stopped when dropped.
#[derive(Debug)]
pub struct MessageDispatcher {
    commands: Sender<DispatcherCommand>,
    /// Next local id to give to a stream.
    next_local_id: AtomicU32,
    reader: Option<JoinHandle<()>>,
}

impl MessageDispatcher {
    /// Start reading messages of `transport`.
    pub(crate) fn start<T: ADBMessageTransport>(transport: T) -> Result<Self> {
        let (commands, commands_receiver) = mpsc::channel();

        let reader = std::thread::Builder::new()
            .name("adb-message-reader".into())
            .spawn(move || {
                MessageReader {
                    transport,
                    commands: commands_receiver,
                    streams: HashMap::new(),
                }
                .run();
            })?;

        Ok(Self {
            commands,
            // Start at a random id, to not match messages of a previous connection
            next_local_id: AtomicU32::new(rand::rng().random_range(1..u32::MAX)),
            reader: Some(reader),
        })
    }

    /// Register a new stream, and return its local id and the channel receiving its messages.
    ///
    /// Messages received for this stream are sent from now on, so register it before sending its `OPEN`.
    pub(crate) fn register(&self) -> Result<(u32, StreamReceiver)> {
        let local_id = match self.next_local_id.fetch_add(1, Ordering::Relaxed) {
            // 0 is not a valid stream id
            0 => self.next_local_id.fetch_add(1, Ordering::Relaxed),
            local_id => local_id,
        };

        let (sender, receiver) = mpsc::channel();
        self.commands
            .send(DispatcherCommand::Register(local_id, sender))
            .map_err(|_| connection_lost_error("message reader stopped"))?;
        Ok((local_id, receiver))
    }

    /// Channel sending commands to the reader thread.
    pub(crate) fn commands(&self) -> Sender<DispatcherCommand> {
        self.commands.clone()
    }

    /// Stop the reader thread, and wait for it to release the transport.
    pub(crate) fn stop(&mut self) {
        let _ = self.commands.send(DispatcherCommand::Stop);
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
            // The device wants to open a stream to us: no service is available on our side
            MessageCommand::Open => log::debug!(
                "refusing stream opened by device to {}",
                String::from_utf8_lossy(message.payload())
            ),
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
}
