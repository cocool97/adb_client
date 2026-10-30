use std::{
    collections::VecDeque,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use crate::{
    Result, RustADBError,
    adb_transport::ADBTransport,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
        adb_transport_message::ADBTransportMessage, message_commands::MessageCommand,
    },
    models::ADBLocalCommand,
};

const REMOTE_ID: u32 = 2002;
const STALE_LOCAL_ID: u32 = 41;
const STALE_REMOTE_ID: u32 = 1001;

/// Called on every message written, to queue the device answers.
type OnWrite = Box<dyn FnMut(&ADBTransportMessage, &mut VecDeque<ADBTransportMessage>) + Send>;

/// Fake transport, answering messages written as a device would.
#[derive(Clone)]
struct FakeTransport {
    reads: Arc<(Mutex<VecDeque<ADBTransportMessage>>, Condvar)>,
    writes: Arc<Mutex<Vec<(MessageCommand, u32, u32)>>>,
    on_write: Arc<Mutex<OnWrite>>,
    /// The connection has been lost: reads fail.
    lost: Arc<AtomicBool>,
}

impl FakeTransport {
    fn new(
        on_write: impl FnMut(&ADBTransportMessage, &mut VecDeque<ADBTransportMessage>) + Send + 'static,
    ) -> Self {
        Self {
            reads: Arc::default(),
            writes: Arc::default(),
            on_write: Arc::new(Mutex::new(Box::new(on_write))),
            lost: Arc::default(),
        }
    }

    fn push_read(&self, message: ADBTransportMessage) {
        let (reads, condvar) = &*self.reads;
        reads.lock().unwrap().push_back(message);
        condvar.notify_all();
    }

    fn lose_connection(&self) {
        self.lost.store(true, Ordering::SeqCst);
        self.reads.1.notify_all();
    }

    fn writes(&self) -> Vec<(MessageCommand, u32, u32)> {
        self.writes.lock().unwrap().clone()
    }
}

impl std::fmt::Debug for FakeTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FakeTransport")
    }
}

impl ADBTransport for FakeTransport {
    fn connect(&mut self) -> Result<()> {
        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }
}

impl ADBMessageTransport for FakeTransport {
    fn read_message_with_timeout(&mut self, read_timeout: Duration) -> Result<ADBTransportMessage> {
        let (reads, condvar) = &*self.reads;
        let (mut reads, _) = condvar
            .wait_timeout_while(reads.lock().unwrap(), read_timeout, |reads| {
                reads.is_empty() && !self.lost.load(Ordering::SeqCst)
            })
            .unwrap();
        if self.lost.load(Ordering::SeqCst) {
            return Err(std::io::Error::from(std::io::ErrorKind::ConnectionReset).into());
        }
        reads.pop_front().ok_or_else(|| {
            RustADBError::IOError(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "no message to read",
            ))
        })
    }

    fn write_message_with_timeout(
        &mut self,
        message: ADBTransportMessage,
        _write_timeout: Duration,
    ) -> Result<()> {
        let header = message.header();
        self.writes
            .lock()
            .unwrap()
            .push((header.command(), header.arg0(), header.arg1()));

        let mut answers = VecDeque::new();
        self.on_write.lock().unwrap()(&message, &mut answers);
        for answer in answers {
            self.push_read(answer);
        }
        Ok(())
    }
}

fn message(command: MessageCommand, arg0: u32, arg1: u32, payload: &[u8]) -> ADBTransportMessage {
    ADBTransportMessage::try_new(command, arg0, arg1, payload).unwrap()
}

/// Return the local id of `message` if it is an `OPEN`.
fn opened_local_id(message: &ADBTransportMessage) -> Option<u32> {
    (message.header().command() == MessageCommand::Open).then(|| message.header().arg0())
}

fn device(transport: &FakeTransport) -> ADBMessageDevice<FakeTransport> {
    ADBMessageDevice::from_connected_transport(transport.clone(), 4096).unwrap()
}

#[test]
fn open_session_skips_messages_of_unknown_streams() {
    let transport = FakeTransport::new(|written, answers| {
        if let Some(local_id) = opened_local_id(written) {
            answers.push_back(message(
                MessageCommand::Okay,
                STALE_REMOTE_ID,
                STALE_LOCAL_ID,
                &[],
            ));
            answers.push_back(message(
                MessageCommand::Write,
                STALE_REMOTE_ID,
                STALE_LOCAL_ID,
                b"stale",
            ));
            answers.push_back(message(
                MessageCommand::Clse,
                STALE_REMOTE_ID,
                STALE_LOCAL_ID,
                &[],
            ));
            answers.push_back(message(MessageCommand::Okay, REMOTE_ID, local_id, &[]));
        }
    });
    let device = device(&transport);

    let session = device.open_session(&ADBLocalCommand::Sync).unwrap();
    assert_eq!(session.remote_id(), REMOTE_ID);

    // Stale OKAY and WRTE are answered by a CLSE, like adb does, stale CLSE is ignored
    assert_eq!(
        transport.writes()[1..],
        [
            (MessageCommand::Clse, 0, STALE_REMOTE_ID),
            (MessageCommand::Clse, 0, STALE_REMOTE_ID),
        ]
    );
}

#[test]
fn open_session_fails_when_device_closes_the_stream() {
    let transport = FakeTransport::new(|written, answers| {
        if let Some(local_id) = opened_local_id(written) {
            answers.push_back(message(MessageCommand::Clse, 0, local_id, &[]));
        }
    });
    let device = device(&transport);

    assert!(device.open_session(&ADBLocalCommand::Sync).is_err());
    // Nothing to close, the stream has never been opened
    assert_eq!(transport.writes().len(), 1);
}

#[test]
fn shell_command_acknowledges_data_only() {
    let transport = FakeTransport::new(|written, answers| {
        if let Some(local_id) = opened_local_id(written) {
            answers.push_back(message(MessageCommand::Okay, REMOTE_ID, local_id, &[]));
            answers.push_back(message(
                MessageCommand::Write,
                STALE_REMOTE_ID,
                STALE_LOCAL_ID,
                b"stale",
            ));
            answers.push_back(message(
                MessageCommand::Write,
                REMOTE_ID,
                local_id,
                b"hello",
            ));
            answers.push_back(message(MessageCommand::Clse, REMOTE_ID, local_id, &[]));
        }
    });
    let device = device(&transport);
    let mut stdout = Vec::new();

    device
        .shell_command(&"echo hello", Some(&mut stdout), None)
        .unwrap();

    assert_eq!(stdout, b"hello");
    let writes = transport.writes();
    let local_id = writes[0].1;
    assert_eq!(
        writes[1..],
        [
            (MessageCommand::Clse, 0, STALE_REMOTE_ID),
            (MessageCommand::Okay, local_id, REMOTE_ID),
        ]
    );
}

#[test]
fn dropping_an_open_session_closes_it() {
    let transport = FakeTransport::new(|written, answers| {
        if let Some(local_id) = opened_local_id(written) {
            answers.push_back(message(MessageCommand::Okay, REMOTE_ID, local_id, &[]));
        }
    });
    let device = device(&transport);

    let session = device.open_session(&ADBLocalCommand::Sync).unwrap();
    let local_id = session.local_id();
    drop(session);

    assert_eq!(
        transport.writes().last(),
        Some(&(MessageCommand::Clse, local_id, REMOTE_ID))
    );
}

#[test]
fn concurrent_sessions_receive_their_own_messages() {
    let transport = FakeTransport::new(|written, answers| {
        if let Some(local_id) = opened_local_id(written) {
            // Each stream gets a remote id derived from its local id
            answers.push_back(message(
                MessageCommand::Okay,
                local_id + 1000,
                local_id,
                &[],
            ));
        }
    });
    let device = device(&transport);

    let mut first = device.open_session(&ADBLocalCommand::Sync).unwrap();
    let mut second = device.open_session(&ADBLocalCommand::Sync).unwrap();
    let (first_id, second_id) = (first.local_id(), second.local_id());

    // Both sessions wait at the same time, messages arrive in the opposite order
    let second_thread = std::thread::spawn(move || second.recv().unwrap().into_payload());
    transport.push_read(message(
        MessageCommand::Write,
        first_id + 1000,
        first_id,
        b"first",
    ));
    transport.push_read(message(
        MessageCommand::Write,
        second_id + 1000,
        second_id,
        b"second",
    ));

    assert_eq!(first.recv().unwrap().into_payload(), b"first");
    assert_eq!(second_thread.join().unwrap(), b"second");
}

#[test]
fn device_opened_streams_are_refused() {
    let transport = FakeTransport::new(|written, answers| {
        if let Some(local_id) = opened_local_id(written) {
            answers.push_back(message(
                MessageCommand::Open,
                STALE_REMOTE_ID,
                0,
                b"tcp:8080\0",
            ));
            answers.push_back(message(MessageCommand::Okay, REMOTE_ID, local_id, &[]));
        }
    });
    let device = device(&transport);

    device.open_session(&ADBLocalCommand::Sync).unwrap();
    assert_eq!(
        transport.writes()[1],
        (MessageCommand::Clse, 0, STALE_REMOTE_ID)
    );
}

#[test]
fn sessions_fail_once_connection_is_lost() {
    let transport = FakeTransport::new(|written, answers| {
        if let Some(local_id) = opened_local_id(written) {
            answers.push_back(message(MessageCommand::Okay, REMOTE_ID, local_id, &[]));
        }
    });
    let device = device(&transport);

    let mut session = device.open_session(&ADBLocalCommand::Sync).unwrap();
    transport.lose_connection();

    let error = session.recv().unwrap_err();
    assert!(
        matches!(&error, RustADBError::IOError(e) if e.kind() == std::io::ErrorKind::ConnectionAborted),
        "{error}"
    );
    assert!(device.open_session(&ADBLocalCommand::Sync).is_err());
}

/// Receive a file from a device sending `payloads` as `WRTE` messages once the session is opened, then closing it.
fn recv_file(payloads: Vec<Vec<u8>>) -> (FakeTransport, Result<Vec<u8>>) {
    let transport = FakeTransport::new(move |written, answers| {
        if let Some(local_id) = opened_local_id(written) {
            answers.push_back(message(MessageCommand::Okay, REMOTE_ID, local_id, &[]));
            for payload in &payloads {
                answers.push_back(message(MessageCommand::Write, REMOTE_ID, local_id, payload));
            }
            answers.push_back(message(MessageCommand::Clse, REMOTE_ID, local_id, &[]));
        }
    });
    let device = device(&transport);

    let mut session = device.open_session(&ADBLocalCommand::Sync).unwrap();
    let mut output = Vec::new();
    let result = session.recv_file(&mut output).map(|()| output);
    (transport, result)
}

/// Encode a sync chunk header.
fn sync_header(id: [u8; 4], length: u32) -> Vec<u8> {
    [id.as_slice(), &length.to_le_bytes()].concat()
}

#[test]
fn recv_file_handles_chunks_split_across_messages() {
    let stream = [
        sync_header(*b"DATA", 6),
        b"hello ".to_vec(),
        sync_header(*b"DATA", 5),
        b"world".to_vec(),
        sync_header(*b"DONE", 0),
    ]
    .concat();
    // Split chunk contents and headers, one message per 3 bytes
    let payloads = stream.chunks(3).map(<[u8]>::to_vec).collect();

    let (transport, output) = recv_file(payloads);

    assert_eq!(output.unwrap(), b"hello world");
    // Every message is acknowledged
    let okays = transport
        .writes()
        .iter()
        .filter(|(command, ..)| *command == MessageCommand::Okay)
        .count();
    assert_eq!(okays, stream.len().div_ceil(3));
}

#[test]
fn recv_file_keeps_content_looking_like_done() {
    let content = sync_header(*b"DONE", 0);
    let payloads = vec![
        [sync_header(*b"DATA", 8), content.clone()].concat(),
        sync_header(*b"DONE", 0),
    ];

    let (_, output) = recv_file(payloads);

    assert_eq!(output.unwrap(), content);
}

#[test]
fn recv_file_returns_fail_reason() {
    let payloads = vec![
        [
            sync_header(*b"FAIL", 14),
            b"No such file".to_vec(),
            b"!!".to_vec(),
        ]
        .concat(),
    ];

    let (_, output) = recv_file(payloads);

    let error = output.unwrap_err();
    assert!(
        matches!(&error, RustADBError::ADBRequestFailed(reason) if reason == "No such file!!"),
        "{error}"
    );
}

#[test]
fn recv_file_fails_when_device_closes_the_session() {
    let (_, output) = recv_file(vec![sync_header(*b"DATA", 10), b"short".to_vec()]);

    assert!(output.is_err());
}
