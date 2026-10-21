use rand::RngExt;
use std::{path::Path, time::Duration};

use crate::{
    Result, RustADBError,
    message_devices::{
        adb_message_transport::ADBMessageTransport,
        adb_session::ADBSession,
        adb_transport_message::{
            ADBTransportMessage, AUTH_RSAPUBLICKEY, AUTH_SIGNATURE, AUTH_TOKEN, MAX_PAYLOAD_SIZE,
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
    /// Maximum message payload size, negotiated with the device during the connection.
    max_payload_size: usize,
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

        Ok(Self {
            transport,
            max_payload_size,
        })
    }

    pub(crate) const fn get_transport_mut(&mut self) -> &mut T {
        &mut self.transport
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

    pub(crate) fn open_synchronization_session(&mut self) -> Result<ADBSession<T>> {
        self.open_session(&ADBLocalCommand::Sync)
    }

    /// Open a new ADB session with the given local command.
    pub(crate) fn open_session(&mut self, cmd: &ADBLocalCommand) -> Result<ADBSession<T>> {
        let mut rng = rand::rng();
        let local_id: u32 = rng.random();

        // adbd used to expect a null-terminated string.
        // keep doing so to maintain compatibility with older versions.
        // https://cs.android.com/android/platform/superproject/+/android-latest-release:packages/modules/adb/sockets.cpp;l=560?q=sockets.cpp
        let mut destination = cmd.to_string().into_bytes();
        if !destination.ends_with(&[0]) {
            destination.push(0);
        }

        let message = ADBTransportMessage::try_new(
            MessageCommand::Open,
            local_id, // Our 'local-id'
            0,
            &destination,
        )?;
        self.transport.write_message(message)?;

        let response = self.transport.read_message()?;

        if response.header().command() != MessageCommand::Okay {
            return Err(RustADBError::ADBRequestFailed(format!(
                "Open session failed: got {} in response instead of OKAY",
                response.header().command()
            )));
        }

        if response.header().arg1() != local_id {
            return Err(RustADBError::ADBRequestFailed(format!(
                "Open session failed: responses used {} for our local_id instead of {local_id}",
                response.header().arg1()
            )));
        }

        Ok(ADBSession::new(
            self.transport.clone(),
            local_id,
            response.header().arg0(),
            self.max_payload_size,
        ))
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
