use super::WebUSBKey;
use crate::message_devices::{
    adb_transport_message::{
        ADBTransportMessage as Message, ADBTransportMessageHeader as Header, AUTH_RSAPUBLICKEY,
        AUTH_SIGNATURE, AUTH_TOKEN,
    },
    message_commands::MessageCommand as Command,
};
use crate::{BinaryDecodable, Result, RustADBError};

// Negotiate the original, checksummed protocol, including one WRTE per OKAY.
// Do not advertise shell_v2 or delayed_ack until implemented here.
const VERSION: u32 = 0x0100_0000;
const MAX_PAYLOAD: usize = 64 * 1024;

pub(super) fn invalid(message: &str) -> RustADBError {
    RustADBError::ADBRequestFailed(message.into())
}

pub(super) trait Transport {
    async fn read(&mut self, output: &mut [u8]) -> Result<usize>;
    async fn write(&mut self, bytes: &[u8]) -> Result<()>;
}

#[derive(Debug)]
pub(super) struct Connection<T> {
    pub(super) transport: T,
    max_payload: usize,
    next_id: u32,
    usable: bool,
}

impl<T: Transport> Connection<T> {
    pub(super) async fn connect(transport: T, key: &WebUSBKey) -> Result<Self> {
        let mut this = Self {
            transport,
            max_payload: MAX_PAYLOAD,
            next_id: 0,
            usable: false,
        };
        this.send(
            Command::Cnxn,
            VERSION,
            u32::try_from(MAX_PAYLOAD)?,
            b"host::\0",
        )
        .await?;
        let mut signed = false;
        let mut offered = false;
        loop {
            let message = this.receive().await?;
            match message.header().command() {
                Command::Cnxn => {
                    if message.header().arg0() != VERSION || message.header().arg1() == 0 {
                        return Err(invalid(
                            "Unsupported ADB connection version or payload limit",
                        ));
                    }
                    this.max_payload = MAX_PAYLOAD.min(message.header().arg1() as usize);
                    this.usable = true;
                    return Ok(this);
                }
                Command::Auth
                    if message.header().arg0() == AUTH_TOKEN && message.payload().len() == 20 =>
                {
                    if !signed {
                        let signature = key.0.sign(message.payload())?;
                        this.send(Command::Auth, AUTH_SIGNATURE, 0, &signature)
                            .await?;
                        signed = true;
                    } else if !offered {
                        let mut public = key.0.android_pubkey_encode()?.into_bytes();
                        public.push(0);
                        this.send(Command::Auth, AUTH_RSAPUBLICKEY, 0, &public)
                            .await?;
                        offered = true;
                    } else {
                        return Err(invalid("ADB host key was not authorized"));
                    }
                }
                _ => return Err(invalid("Expected ADB CNXN or authentication token")),
            }
        }
    }

    async fn exact(&mut self, bytes: &mut [u8]) -> Result<()> {
        let mut offset = 0;
        while offset < bytes.len() {
            let n = self.transport.read(&mut bytes[offset..]).await?;
            if n > bytes.len() - offset {
                return Err(invalid("USB read exceeded requested length"));
            }
            // USB zero-length packets terminate a transfer; they are not EOF.
            offset += n;
        }
        Ok(())
    }

    async fn receive(&mut self) -> Result<Message> {
        let mut bytes = [0; 24];
        self.exact(&mut bytes).await?;
        let command = u32::from_le_bytes(bytes[..4].try_into()?);
        if u32::from_le_bytes(bytes[20..].try_into()?) != !command {
            return Err(invalid("Invalid ADB header magic"));
        }
        let header = Header::decode(&bytes)?;
        if header.data_length() as usize > MAX_PAYLOAD {
            return Err(invalid("ADB message exceeds advertised payload limit"));
        }
        let mut payload = vec![0; header.data_length() as usize];
        self.exact(&mut payload).await?;
        let message = Message::from_header_and_payload(header, payload);
        if !message.check_message_integrity() {
            return Err(RustADBError::InvalidIntegrity(
                message.header().data_crc32(),
                Header::compute_crc32(message.payload()),
            ));
        }
        Ok(message)
    }

    async fn send(&mut self, command: Command, arg0: u32, arg1: u32, bytes: &[u8]) -> Result<()> {
        let message = Message::try_new(command, arg0, arg1, bytes)?;
        self.transport.write(&message.header().as_bytes()).await?;
        if !bytes.is_empty() {
            self.transport.write(bytes).await?;
        }
        Ok(())
    }

    pub(super) async fn open(&mut self, service: &str) -> Result<Stream<'_, T>> {
        if !self.usable {
            return Err(invalid(
                "ADB connection must be disconnected and reconnected",
            ));
        }
        if service.is_empty() || service.contains('\0') || service.len() >= self.max_payload {
            return Err(invalid("Invalid or oversized ADB service name"));
        }
        // Leaving this false on an error or dropped future prevents reusing a
        // connection with an incomplete packet or unclosed logical stream.
        self.usable = false;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| invalid("ADB stream IDs exhausted"))?;
        let local = self.next_id;
        let mut destination = service.as_bytes().to_vec();
        destination.push(0);
        self.send(Command::Open, local, 0, &destination).await?;
        let response = self.receive().await?;
        response.assert_command(Command::Okay)?;
        if response.header().arg1() != local
            || response.header().arg0() == 0
            || !response.payload().is_empty()
        {
            return Err(invalid("ADB service open failed"));
        }
        let remote = response.header().arg0();
        Ok(Stream {
            connection: self,
            local,
            remote,
            pending: None,
            ready: true,
            closed: false,
        })
    }
}

#[derive(Debug)]
pub(super) struct Stream<'a, T> {
    connection: &'a mut Connection<T>,
    local: u32,
    remote: u32,
    pending: Option<Vec<u8>>,
    ready: bool,
    closed: bool,
}

impl<T: Transport> Stream<'_, T> {
    fn begin(&mut self) -> Result<()> {
        if !self.ready || self.closed {
            return Err(invalid(
                "ADB stream is closed or an earlier operation was cancelled",
            ));
        }
        self.ready = false;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Message> {
        let message = self.connection.receive().await?;
        if message.header().arg0() != self.remote || message.header().arg1() != self.local {
            return Err(invalid("ADB stream ID mismatch"));
        }
        Ok(message)
    }

    async fn acknowledge(&mut self) -> Result<()> {
        self.connection
            .send(Command::Okay, self.local, self.remote, &[])
            .await
    }

    pub(super) async fn read(&mut self) -> Result<Option<Vec<u8>>> {
        if self.closed {
            return Ok(None);
        }
        self.begin()?;
        let message = if let Some(bytes) = self.pending.take() {
            self.acknowledge().await?;
            self.ready = true;
            return Ok(Some(bytes));
        } else {
            self.receive().await?
        };
        match message.header().command() {
            Command::Write => {
                self.acknowledge().await?;
                self.ready = true;
                Ok(Some(message.into_payload()))
            }
            Command::Clse if message.payload().is_empty() => {
                self.connection
                    .send(Command::Clse, self.local, self.remote, &[])
                    .await?;
                self.closed = true;
                self.connection.usable = true;
                Ok(None)
            }
            _ => Err(invalid("Expected ADB stream data or close")),
        }
    }

    pub(super) async fn write(&mut self, bytes: &[u8]) -> Result<()> {
        self.begin()?;
        for chunk in bytes.chunks(self.connection.max_payload) {
            self.connection
                .send(Command::Write, self.local, self.remote, chunk)
                .await?;
            loop {
                let response = self.receive().await?;
                match response.header().command() {
                    Command::Okay if response.payload().is_empty() => break,
                    Command::Write if self.pending.is_none() => {
                        // Retain one packet without acknowledging it. Legacy
                        // flow control prevents a second packet until read().
                        self.pending = Some(response.into_payload());
                    }
                    _ => return Err(invalid("Expected ADB write acknowledgement")),
                }
            }
        }
        self.ready = true;
        Ok(())
    }

    pub(super) async fn close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        self.begin()?;
        self.connection
            .send(Command::Clse, self.local, self.remote, &[])
            .await?;
        loop {
            let response = self.receive().await?;
            match response.header().command() {
                Command::Clse if response.payload().is_empty() => break,
                Command::Write => {} // A packet may have crossed our close.
                _ => return Err(invalid("Expected ADB close acknowledgement")),
            }
        }
        self.pending = None;
        self.closed = true;
        self.connection.usable = true;
        Ok(())
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
