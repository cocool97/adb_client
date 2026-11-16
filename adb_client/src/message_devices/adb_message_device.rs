use std::{
    collections::HashMap,
    path::Path,
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use crate::{
    Result, RustADBError,
    message_devices::{
        adb_message_transport::ADBMessageTransport,
        adb_session::{ADBSession, ADBSessionOpener},
        adb_transport_message::{
            ADBTransportMessage, AUTH_RSAPUBLICKEY, AUTH_SIGNATURE, AUTH_TOKEN, MAX_PAYLOAD_SIZE,
        },
        commands::ForwardListener,
        message_commands::MessageCommand,
        message_dispatcher::MessageDispatcher,
        models::{ADBRsaKey, read_adb_private_key},
    },
    models::ADBLocalCommand,
};

/// Generic structure representing an ADB device reachable over an [`ADBMessageTransport`].
/// Structure is totally agnostic over which transport is truly used.
#[derive(Debug)]
pub struct ADBMessageDevice<T: ADBMessageTransport> {
    transport: T,
    /// Maximum message payload size, negotiated with the device during the connection.
    max_payload_size: usize,
    /// Reads every message received, and routes it to the session it belongs to.
    dispatcher: MessageDispatcher,
    /// Host listeners of forwards, by host port. They live as long as the device.
    forwards: Mutex<HashMap<u16, ForwardListener>>,
}

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    /// Instantiate a new [`ADBMessageDevice`]
    pub fn new<P: AsRef<Path>>(mut transport: T, adb_private_key_path: P) -> Result<Self> {
        let private_key = if let Some(private_key) = read_adb_private_key(&adb_private_key_path)? {
            private_key
        } else {
            log::warn!(
                "No private key found at path {}. Generating a new random.",
                adb_private_key_path.as_ref().display()
            );
            ADBRsaKey::new_random()?
        };

        let max_payload_size = match Self::connect(&mut transport, &private_key) {
            Ok(max_payload_size) => max_payload_size,
            Err(e) => {
                // Best effort here
                let _ = transport.disconnect();
                return Err(e);
            }
        };

        Self::from_connected_transport(transport.clone(), max_payload_size).inspect_err(|_| {
            // Best effort here
            let _ = transport.disconnect();
        })
    }

    /// Instantiate a new [`ADBMessageDevice`] from a transport on which the connection handshake is done, and
    /// start reading its messages.
    pub(crate) fn from_connected_transport(transport: T, max_payload_size: usize) -> Result<Self> {
        let dispatcher = MessageDispatcher::start(transport.clone(), max_payload_size)?;
        Ok(Self {
            transport,
            max_payload_size,
            dispatcher,
            forwards: Mutex::default(),
        })
    }

    pub(crate) const fn transport(&self) -> &T {
        &self.transport
    }

    /// Send initial connect, and return the negotiated maximum payload size
    fn connect(transport: &mut T, private_key: &ADBRsaKey) -> Result<usize> {
        transport.connect()?;

        let message = ADBTransportMessage::try_new(
            MessageCommand::Cnxn,
            0x0100_0000,
            MAX_PAYLOAD_SIZE,
            format!("host::{}\0", env!("CARGO_PKG_NAME")).as_bytes(),
        )?;

        transport.write_message(message)?;

        let message = Self::read_handshake_message(transport, None)?;

        // Check if a client is requesting a secure connection and upgrade it if necessary
        match message.header().command() {
            MessageCommand::Stls => {
                transport.write_message(ADBTransportMessage::try_new(
                    MessageCommand::Stls,
                    1,
                    0,
                    &[],
                )?)?;
                transport.upgrade_connection()?;
                log::debug!("Connection successfully upgraded from TCP to TLS");

                let message = Self::read_handshake_message(transport, None)?;
                Self::handle_connection(&message)
            }
            MessageCommand::Cnxn => {
                log::debug!("Unencrypted connection established");
                Self::handle_connection(&message)
            }
            MessageCommand::Auth => {
                log::debug!("Authentication required");
                Self::auth_handshake(transport, message, private_key)
            }
            _ => Err(crate::RustADBError::WrongResponseReceived(
                "Expected CNXN, STLS or AUTH command".to_string(),
                message.header().command().to_string(),
            )),
        }
    }

    /// Read the next handshake message (`CNXN`, `AUTH` or `STLS`).
    ///
    /// Stream messages (`OKAY`, `WRTE`, `CLSE`) can still be in flight from a previous connection that was not
    /// properly closed.
    fn read_handshake_message(
        transport: &mut T,
        timeout: Option<Duration>,
    ) -> Result<ADBTransportMessage> {
        loop {
            let message = match timeout {
                Some(timeout) => transport.read_message_with_timeout(timeout)?,
                None => transport.read_message()?,
            };

            match message.header().command() {
                MessageCommand::Okay | MessageCommand::Write | MessageCommand::Clse => {
                    log::debug!(
                        "discarding stale {} message received during handshake",
                        message.header().command()
                    );
                }
                _ => return Ok(message),
            }
        }
    }

    /// Authenticate with `private_key`, and return the negotiated maximum payload size
    fn auth_handshake(
        transport: &mut T,
        message: ADBTransportMessage,
        private_key: &ADBRsaKey,
    ) -> Result<usize> {
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

        transport.write_message(message)?;

        let received_response = Self::read_handshake_message(transport, None)?;

        if received_response.header().command() == MessageCommand::Cnxn {
            log::info!("Authentication OK");
            return Self::handle_connection(&received_response);
        }

        let mut pubkey = private_key.android_pubkey_encode()?.into_bytes();
        pubkey.push(b'\0');

        let message =
            ADBTransportMessage::try_new(MessageCommand::Auth, AUTH_RSAPUBLICKEY, 0, &pubkey)?;

        transport.write_message(message)?;

        let response = Self::read_handshake_message(transport, Some(Duration::from_secs(10)))?;

        log::info!("Authentication OK");
        Self::handle_connection(&response)
    }

    /// Handle the `CNXN` message ending the connection handshake, and return the negotiated maximum payload size.
    fn handle_connection(message: &ADBTransportMessage) -> Result<usize> {
        message.assert_command(MessageCommand::Cnxn)?;

        // The device answers with the minimum between our maximum payload size and its own
        let max_payload = message.header().arg1().min(MAX_PAYLOAD_SIZE) as usize;
        log::debug!(
            "device info {}, maximum payload size {max_payload}",
            String::from_utf8_lossy(message.payload()),
        );

        Ok(max_payload)
    }

    pub(crate) fn open_synchronization_session(&self) -> Result<ADBSession<T>> {
        self.open_session(&ADBLocalCommand::Sync)
    }

    /// Open a new ADB session with the given local command.
    pub(crate) fn open_session(&self, cmd: &ADBLocalCommand) -> Result<ADBSession<T>> {
        self.session_opener().open(cmd)
    }

    /// Return an opener of sessions on this device, usable from other threads.
    pub(crate) fn session_opener(&self) -> ADBSessionOpener<T> {
        ADBSessionOpener::new(
            self.transport.clone(),
            self.dispatcher.handle().clone(),
            self.max_payload_size,
        )
    }

    pub(crate) const fn dispatcher(&self) -> &MessageDispatcher {
        &self.dispatcher
    }

    /// Host listeners of forwards, by host port.
    pub(crate) fn forwards(&self) -> Result<MutexGuard<'_, HashMap<u16, ForwardListener>>> {
        self.forwards.lock().map_err(|_| RustADBError::PoisonError)
    }
}

impl<T: ADBMessageTransport> Drop for ADBMessageDevice<T> {
    fn drop(&mut self) {
        // Stop accepting forwarded connections, before the sessions they would open fail
        if let Ok(forwards) = self.forwards.get_mut() {
            forwards.clear();
        }
        // The reader must release the transport before it gets disconnected
        self.dispatcher.stop();
        // Best effort here
        let _ = self.transport.disconnect();
    }
}
