use rand::RngExt;
use std::{path::Path, time::Duration};

use crate::{
    Result, RustADBError,
    message_devices::{
        adb_message_transport::ADBMessageTransport,
        adb_session::ADBSession,
        adb_transport_message::{
            ADBTransportMessage, AUTH_RSAPUBLICKEY, AUTH_SIGNATURE, AUTH_TOKEN,
        },
        message_commands::{MessageCommand, MessageSubcommand},
        models::{ADBRsaKey, read_adb_private_key},
        utils::BinaryEncodable,
    },
    models::ADBLocalCommand,
};

/// Generic structure representing an ADB device reachable over an [`ADBMessageTransport`].
/// Structure is totally agnostic over which transport is truly used.
#[derive(Debug)]
pub struct ADBMessageDevice<T: ADBMessageTransport> {
    transport: T,
}

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    /// Instantiate a new [`ADBMessageTransport`]
    pub fn new<P: AsRef<Path>>(transport: T, adb_private_key_path: P) -> Result<Self> {
        let private_key = if let Some(private_key) = read_adb_private_key(&adb_private_key_path)? {
            private_key
        } else {
            log::warn!(
                "No private key found at path {}. Generating a new random.",
                adb_private_key_path.as_ref().display()
            );
            ADBRsaKey::new_random()?
        };

        let mut message_device = Self { transport };
        message_device.connect(&private_key)?;

        Ok(message_device)
    }

    pub(crate) const fn get_transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Send initial connect
    fn connect(&mut self, private_key: &ADBRsaKey) -> Result<()> {
        self.get_transport_mut().connect()?;

        let message = ADBTransportMessage::try_new(
            MessageCommand::Cnxn,
            0x0100_0000,
            1_048_576,
            format!("host::{}\0", env!("CARGO_PKG_NAME")).as_bytes(),
        )?;

        self.get_transport_mut().write_message(message)?;

        let message = self.get_transport_mut().read_message()?;

        // Check if a client is requesting a secure connection and upgrade it if necessary
        match message.header().command() {
            MessageCommand::Stls => {
                self.get_transport_mut()
                    .write_message(ADBTransportMessage::try_new(
                        MessageCommand::Stls,
                        1,
                        0,
                        &[],
                    )?)?;
                self.get_transport_mut().upgrade_connection()?;
                log::debug!("Connection successfully upgraded from TCP to TLS");
                Ok(())
            }
            MessageCommand::Cnxn => {
                log::debug!("Unencrypted connection established");
                Ok(())
            }
            MessageCommand::Auth => {
                log::debug!("Authentication required");
                self.auth_handshake(message, private_key)
            }
            _ => Err(crate::RustADBError::WrongResponseReceived(
                "Expected CNXN, STLS or AUTH command".to_string(),
                message.header().command().to_string(),
            )),
        }
    }

    fn auth_handshake(
        &mut self,
        message: ADBTransportMessage,
        private_key: &ADBRsaKey,
    ) -> Result<()> {
        match message.header().command() {
            MessageCommand::Auth => {
                log::debug!("Authentication required");
            }
            _ => return Ok(()),
        }

        // At this point, we should have received an AUTH message with arg0 == 1
        let auth_message = match message.header().arg0() {
            AUTH_TOKEN => message,
            v => {
                return Err(RustADBError::ADBRequestFailed(format!(
                    "Received AUTH message with type != 1 ({v})"
                )));
            }
        };

        let sign = private_key.sign(auth_message.into_payload())?;

        let message = ADBTransportMessage::try_new(MessageCommand::Auth, AUTH_SIGNATURE, 0, &sign)?;

        self.transport.write_message(message)?;

        let received_response = self.transport.read_message()?;

        if received_response.header().command() == MessageCommand::Cnxn {
            log::info!(
                "Authentication OK, device info {}",
                String::from_utf8(received_response.into_payload())?
            );
            return Ok(());
        }

        let mut pubkey = private_key.android_pubkey_encode()?.into_bytes();
        pubkey.push(b'\0');

        let message =
            ADBTransportMessage::try_new(MessageCommand::Auth, AUTH_RSAPUBLICKEY, 0, &pubkey)?;

        self.transport.write_message(message)?;

        let response = self
            .transport
            .read_message_with_timeout(Duration::from_secs(10))
            .and_then(|message| {
                message.assert_command(MessageCommand::Cnxn)?;
                Ok(message)
            })?;

        log::info!(
            "Authentication OK, device info {}",
            String::from_utf8(response.into_payload())?
        );
        Ok(())
    }

    pub(crate) fn open_synchronization_session(&mut self) -> Result<ADBSession<T>> {
        self.open_session(&ADBLocalCommand::Sync)
    }

    pub(crate) fn open_session(&mut self, cmd: &ADBLocalCommand) -> Result<ADBSession<T>> {
        let mut rng = rand::rng();
        let local_id: u32 = rng.random();

        let message = ADBTransportMessage::try_new(
            MessageCommand::Open,
            local_id, // Our 'local-id'
            0,
            cmd.to_string().as_bytes(),
        )?;
        self.transport.write_message(message)?;

        loop {
            let response = self.transport.read_message()?;

            match response.header().command() {
                MessageCommand::Okay => {
                    if response.header().arg1() != local_id {
                        log::debug!(
                            "Ignoring stale OKAY for local_id {} while opening local_id {local_id}",
                            response.header().arg1()
                        );
                        continue;
                    }

                    return Ok(ADBSession::new(
                        self.transport.clone(),
                        local_id,
                        response.header().arg0(),
                    ));
                }
                MessageCommand::Clse => {
                    if response.header().arg1() == local_id {
                        return Err(RustADBError::ADBRequestFailed(format!(
                            "Open session failed: device closed stream for local_id {local_id}"
                        )));
                    }

                    log::debug!(
                        "Ignoring stale CLSE for local_id {} while opening local_id {local_id}",
                        response.header().arg1()
                    );
                }
                MessageCommand::Write => {
                    let stale_remote_id = response.header().arg0();
                    let stale_local_id = response.header().arg1();

                    if stale_local_id == local_id {
                        return Err(RustADBError::ADBRequestFailed(format!(
                            "Open session failed: got WRTE before OKAY for local_id {local_id}"
                        )));
                    }

                    log::debug!(
                        "Acknowledging and discarding stale WRTE for local_id {stale_local_id} while opening local_id {local_id}",
                    );

                    self.transport.write_message(ADBTransportMessage::try_new(
                        MessageCommand::Okay,
                        stale_local_id,
                        stale_remote_id,
                        &[],
                    )?)?;
                }
                command => {
                    return Err(RustADBError::ADBRequestFailed(format!(
                        "Open session failed: got {command} in response instead of OKAY"
                    )));
                }
            }
        }
    }

    pub(crate) fn end_transaction(&mut self, session: &mut ADBSession<T>) -> Result<()> {
        let quit_buffer = MessageSubcommand::Quit.with_arg(0u32);
        session.send_and_expect_okay(ADBTransportMessage::try_new(
            MessageCommand::Write,
            session.local_id(),
            session.remote_id(),
            &quit_buffer.encode(),
        )?)?;

        let _discard_close = self.transport.read_message()?;
        Ok(())
    }
}

impl<T: ADBMessageTransport> Drop for ADBMessageDevice<T> {
    fn drop(&mut self) {
        // Best effort here
        let _ = self.get_transport_mut().disconnect();
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use crate::{
        RustADBError,
        adb_transport::ADBTransport,
        message_devices::{
            adb_message_transport::ADBMessageTransport, adb_transport_message::ADBTransportMessage,
            message_commands::MessageCommand,
        },
        models::ADBLocalCommand,
    };

    use super::ADBMessageDevice;

    #[derive(Clone, Default)]
    struct FakeTransport {
        reads: Arc<Mutex<VecDeque<ADBTransportMessage>>>,
        writes: Arc<Mutex<Vec<(MessageCommand, u32, u32, Vec<u8>)>>>,
        on_open: Option<fn(u32, &mut VecDeque<ADBTransportMessage>)>,
    }

    impl FakeTransport {
        fn with_open_response(on_open: fn(u32, &mut VecDeque<ADBTransportMessage>)) -> Self {
            Self {
                reads: Arc::new(Mutex::new(VecDeque::new())),
                writes: Arc::new(Mutex::new(Vec::new())),
                on_open: Some(on_open),
            }
        }
    }

    impl ADBTransport for FakeTransport {
        fn connect(&mut self) -> crate::Result<()> {
            Ok(())
        }

        fn disconnect(&mut self) -> crate::Result<()> {
            Ok(())
        }
    }

    impl ADBMessageTransport for FakeTransport {
        fn read_message_with_timeout(
            &mut self,
            _read_timeout: Duration,
        ) -> crate::Result<ADBTransportMessage> {
            self.reads
                .lock()
                .expect("reads mutex poisoned")
                .pop_front()
                .ok_or_else(|| {
                    RustADBError::ADBRequestFailed("unexpected end of test stream".to_string())
                })
        }

        fn write_message_with_timeout(
            &mut self,
            message: ADBTransportMessage,
            _write_timeout: Duration,
        ) -> crate::Result<()> {
            let header = message.header();
            let command = header.command();
            let arg0 = header.arg0();
            let arg1 = header.arg1();
            let payload = message.into_payload();

            if command == MessageCommand::Open
                && let Some(on_open) = self.on_open.take()
            {
                on_open(arg0, &mut self.reads.lock().expect("reads mutex poisoned"));
            }

            self.writes
                .lock()
                .expect("writes mutex poisoned")
                .push((command, arg0, arg1, payload));
            Ok(())
        }
    }

    fn queue_open_session_stale_frames(local_id: u32, reads: &mut VecDeque<ADBTransportMessage>) {
        let stale_local_id = 41;
        let stale_remote_id = 1001;
        let expected_remote_id = 2002;

        reads.push_back(
            ADBTransportMessage::try_new(
                MessageCommand::Okay,
                stale_remote_id,
                stale_local_id,
                &[],
            )
            .expect("stale OKAY message"),
        );
        reads.push_back(
            ADBTransportMessage::try_new(
                MessageCommand::Clse,
                stale_remote_id,
                stale_local_id,
                &[],
            )
            .expect("stale CLSE message"),
        );
        reads.push_back(
            ADBTransportMessage::try_new(MessageCommand::Okay, expected_remote_id, local_id, &[])
                .expect("matching OKAY message"),
        );
    }

    fn queue_open_session_stale_write_frame(
        local_id: u32,
        reads: &mut VecDeque<ADBTransportMessage>,
    ) {
        let stale_local_id = 41;
        let stale_remote_id = 1001;
        let expected_remote_id = 2002;

        reads.push_back(
            ADBTransportMessage::try_new(
                MessageCommand::Write,
                stale_remote_id,
                stale_local_id,
                b"stale",
            )
            .expect("stale WRTE message"),
        );
        reads.push_back(
            ADBTransportMessage::try_new(MessageCommand::Okay, expected_remote_id, local_id, &[])
                .expect("matching OKAY message"),
        );
    }

    fn queue_shell_command_frames(local_id: u32, reads: &mut VecDeque<ADBTransportMessage>) {
        let remote_id = 2002;

        reads.push_back(
            ADBTransportMessage::try_new(MessageCommand::Okay, remote_id, local_id, &[])
                .expect("session OKAY message"),
        );
        reads.push_back(
            ADBTransportMessage::try_new(MessageCommand::Write, remote_id, local_id, b"hello")
                .expect("shell WRTE message"),
        );
        reads.push_back(
            ADBTransportMessage::try_new(MessageCommand::Clse, remote_id, local_id, &[])
                .expect("shell CLSE message"),
        );
    }

    fn queue_shell_command_stale_write_frame(
        local_id: u32,
        reads: &mut VecDeque<ADBTransportMessage>,
    ) {
        let stale_local_id = 41;
        let stale_remote_id = 1001;
        let remote_id = 2002;

        reads.push_back(
            ADBTransportMessage::try_new(MessageCommand::Okay, remote_id, local_id, &[])
                .expect("session OKAY message"),
        );
        reads.push_back(
            ADBTransportMessage::try_new(
                MessageCommand::Write,
                stale_remote_id,
                stale_local_id,
                b"stale",
            )
            .expect("stale shell WRTE message"),
        );
        reads.push_back(
            ADBTransportMessage::try_new(MessageCommand::Write, remote_id, local_id, b"hello")
                .expect("shell WRTE message"),
        );
        reads.push_back(
            ADBTransportMessage::try_new(MessageCommand::Clse, remote_id, local_id, &[])
                .expect("shell CLSE message"),
        );
    }

    #[test]
    fn open_session_ignores_stale_okay_and_clse_frames() {
        let expected_remote_id = 2002;
        let transport = FakeTransport::with_open_response(queue_open_session_stale_frames);
        let mut device = ADBMessageDevice { transport };

        let session = device
            .open_session(&ADBLocalCommand::Sync)
            .expect("session should open after ignoring stale frames");

        assert_eq!(session.remote_id(), expected_remote_id);
    }

    #[test]
    fn open_session_acknowledges_and_discards_stale_write_frames() {
        let expected_remote_id = 2002;
        let transport = FakeTransport::with_open_response(queue_open_session_stale_write_frame);
        let writes = transport.writes.clone();
        let mut device = ADBMessageDevice { transport };

        let session = device
            .open_session(&ADBLocalCommand::Sync)
            .expect("session should open after stale WRTE");

        assert_eq!(session.remote_id(), expected_remote_id);

        let writes = writes.lock().expect("writes mutex poisoned");
        assert_eq!(writes.len(), 2, "expected OPEN and stale WRTE ack");
        assert_eq!(writes[0].0, MessageCommand::Open);
        assert_eq!(writes[1].0, MessageCommand::Okay);
        assert_eq!(writes[1].1, 41);
        assert_eq!(writes[1].2, 1001);
    }

    #[test]
    fn shell_command_does_not_acknowledge_close_messages() {
        let remote_id = 2002;
        let transport = FakeTransport::with_open_response(queue_shell_command_frames);
        let writes = transport.writes.clone();
        let mut device = ADBMessageDevice { transport };
        let mut stdout = Vec::new();

        device
            .shell_command(&"echo hello", Some(&mut stdout), None)
            .expect("shell command should succeed");

        assert_eq!(stdout, b"hello");

        let writes = writes.lock().expect("writes mutex poisoned");
        assert_eq!(writes.len(), 2, "expected OPEN and WRTE ack only");
        assert_eq!(writes[0].0, MessageCommand::Open);
        assert_eq!(writes[1].0, MessageCommand::Okay);
        assert_eq!(writes[1].1, writes[0].1);
        assert_eq!(writes[1].2, remote_id);
    }

    #[test]
    fn shell_command_acknowledges_and_discards_stale_write_frames() {
        let remote_id = 2002;
        let transport = FakeTransport::with_open_response(queue_shell_command_stale_write_frame);
        let writes = transport.writes.clone();
        let mut device = ADBMessageDevice { transport };
        let mut stdout = Vec::new();

        device
            .shell_command(&"echo hello", Some(&mut stdout), None)
            .expect("shell command should succeed");

        assert_eq!(stdout, b"hello");

        let writes = writes.lock().expect("writes mutex poisoned");
        assert_eq!(
            writes.len(),
            3,
            "expected OPEN, stale WRTE ack, and current WRTE ack"
        );
        assert_eq!(writes[0].0, MessageCommand::Open);
        assert_eq!(writes[1].0, MessageCommand::Okay);
        assert_eq!(writes[1].1, 41);
        assert_eq!(writes[1].2, 1001);
        assert_eq!(writes[2].0, MessageCommand::Okay);
        assert_eq!(writes[2].1, writes[0].1);
        assert_eq!(writes[2].2, remote_id);
    }
}
