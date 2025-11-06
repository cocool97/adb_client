use super::{ADBRsaKey, ADBTransportMessage, MessageCommand, models::MessageSubcommand};
use crate::device::adb_transport_message::{AUTH_RSAPUBLICKEY, AUTH_SIGNATURE, AUTH_TOKEN};
use crate::{ADBMessageTransport, AdbStatResponse, Result, RustADBError, constants::BUFFER_SIZE};
use bincode::config::{Configuration, Fixint, LittleEndian, NoLimit};
use byteorder::ReadBytesExt;
use rand::Rng;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::io::{Cursor, Read, Seek};
use std::time::Duration;

const BINCODE_CONFIG: Configuration<LittleEndian, Fixint, NoLimit> = bincode::config::legacy();

pub(crate) fn bincode_serialize_to_vec<E: Serialize>(val: E) -> Result<Vec<u8>> {
    bincode::serde::encode_to_vec(val, BINCODE_CONFIG).map_err(|_e| RustADBError::ConversionError)
}

pub(crate) fn bincode_deserialize_from_slice<D: DeserializeOwned>(data: &[u8]) -> Result<D> {
    let (response, _) = bincode::serde::decode_from_slice(data, BINCODE_CONFIG)
        .map_err(|_e| RustADBError::ConversionError)?;

    Ok(response)
}

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

    pub(crate) fn auth_handshake(
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

        let message = ADBTransportMessage::new(MessageCommand::Auth, AUTH_SIGNATURE, 0, &sign);

        self.get_transport_mut().write_message(message)?;

        let received_response = self.get_transport_mut().read_message()?;

        if received_response.header().command() == MessageCommand::Cnxn {
            log::info!(
                "Authentication OK, device info {}",
                String::from_utf8(received_response.into_payload())?
            );
            return Ok(());
        }

        let mut pubkey = private_key.android_pubkey_encode()?.into_bytes();
        pubkey.push(b'\0');

        let message = ADBTransportMessage::new(MessageCommand::Auth, AUTH_RSAPUBLICKEY, 0, &pubkey);

        self.get_transport_mut().write_message(message)?;

        let response = self
            .get_transport_mut()
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
                        len.replace(u64::from(rdr.read_u32::<byteorder::LittleEndian>()?));
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
                .read_u32::<byteorder::LittleEndian>()?
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
        let mut buffer = vec![0; BUFFER_SIZE].into_boxed_slice();
        let amount_read = reader.read(&mut buffer)?;
        let subcommand_data = MessageSubcommand::Data.with_arg(u32::try_from(amount_read)?);

        let mut serialized_message = bincode_serialize_to_vec(&subcommand_data)?;
        serialized_message.append(&mut buffer[..amount_read].to_vec());

        let message = ADBTransportMessage::new(
            MessageCommand::Write,
            local_id,
            remote_id,
            &serialized_message,
        );

        self.send_and_expect_okay(message)?;

        loop {
            let mut buffer = vec![0; BUFFER_SIZE].into_boxed_slice();

            match reader.read(&mut buffer) {
                Ok(0) => {
                    // Currently file mtime is not forwarded
                    let subcommand_data = MessageSubcommand::Done.with_arg(0);

                    let serialized_message = bincode_serialize_to_vec(&subcommand_data)?;
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
                                "Wrong command received {c}"
                            )));
                        }
                    }
                }
                Ok(size) => {
                    let subcommand_data = MessageSubcommand::Data.with_arg(u32::try_from(size)?);

                    let mut serialized_message = bincode_serialize_to_vec(&subcommand_data)?;
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
        let stat_buffer = MessageSubcommand::Stat.with_arg(u32::try_from(remote_path.len())?);
        let message = ADBTransportMessage::new(
            MessageCommand::Write,
            self.get_local_id()?,
            self.get_remote_id()?,
            &bincode_serialize_to_vec(&stat_buffer)?,
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

        bincode_deserialize_from_slice(&response.into_payload()[4..])
    }

    pub(crate) fn end_transaction(&mut self) -> Result<()> {
        let quit_buffer = MessageSubcommand::Quit.with_arg(0u32);
        self.send_and_expect_okay(ADBTransportMessage::new(
            MessageCommand::Write,
            self.get_local_id()?,
            self.get_remote_id()?,
            &bincode_serialize_to_vec(&quit_buffer)?,
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
