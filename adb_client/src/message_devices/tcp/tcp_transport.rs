use rcgen::{CertificateParams, KeyPair, PKCS_RSA_SHA256};
use rustls::{
    ClientConfig, ClientConnection, KeyLogFile, SignatureScheme,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, PrivatePkcs8KeyDer, pem::PemObject},
};

use crate::{
    Result, RustADBError,
    adb_transport::ADBTransport,
    message_devices::{
        adb_message_transport::{ADBMessageTransport, MESSAGE_END_TIMEOUT},
        adb_transport_message::{ADBTransportMessage, ADBTransportMessageHeader},
    },
};
use std::{
    fs::read_to_string,
    io::{ErrorKind, Read, Write},
    net::{Shutdown, SocketAddr, TcpStream},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

/// Maximum size of a TLS record on the wire.
const MAX_TLS_RECORD_SIZE: usize = 16 * 1024 + 2048;

/// A TCP connection, possibly upgraded to TLS.
///
/// Reading and writing use their own handle on the socket, so a blocking read does not prevent writing.
#[derive(Debug)]
struct Connection {
    /// Socket handle used to read. Only one message can be read at a time.
    reader: Mutex<TcpStream>,
    /// Socket handle used to write.
    writer: Mutex<TcpStream>,
    /// TLS state, once the connection has been upgraded.
    tls: OnceLock<SharedTls>,
}

impl Connection {
    fn new(stream: TcpStream) -> Result<Self> {
        Ok(Self {
            reader: Mutex::new(stream.try_clone()?),
            writer: Mutex::new(stream),
            tls: OnceLock::new(),
        })
    }

    fn read_message(&self, read_timeout: Duration) -> Result<ADBTransportMessage> {
        let (header, msg_data) = {
            let socket = self.reader.lock()?;
            match self.tls.get() {
                Some(tls) => {
                    read_header_and_payload(&mut tls.reader(&socket), &socket, read_timeout)?
                }
                None => read_header_and_payload(&mut &*socket, &socket, read_timeout)?,
            }
        };

        let message = ADBTransportMessage::from_header_and_payload(header, msg_data);

        // Check message integrity
        if !message.payload().is_empty() && !message.check_message_integrity() {
            return Err(RustADBError::InvalidIntegrity(
                ADBTransportMessageHeader::compute_crc32(message.payload()),
                message.header().data_crc32(),
            ));
        }

        Ok(message)
    }

    fn write_message(&self, message: &ADBTransportMessage, write_timeout: Duration) -> Result<()> {
        let mut data = message.header().as_bytes();
        data.extend_from_slice(message.payload());

        let mut socket = self.writer.lock()?;
        socket.set_write_timeout(Some(write_timeout))?;

        if let Some(tls) = self.tls.get() {
            // Encrypt while holding the socket, so records are written in the order they are encrypted
            data = tls.encrypt(&data)?;
        }

        socket.write_all(&data)?;
        Ok(socket.flush()?)
    }

    fn upgrade(&self, tls: ClientConnection) -> Result<()> {
        let already_upgraded =
            || RustADBError::UpgradeError("cannot upgrade a TLS connection...".into());

        let mut socket = self.writer.lock()?;
        if self.tls.get().is_some() {
            return Err(already_upgraded());
        }

        // Nothing else uses the connection during the connection handshake. The TLS state is set while still
        // holding the socket, so a concurrent upgrade sees it.
        self.tls
            .set(SharedTls::handshake(tls, &mut socket)?)
            .map_err(|_| already_upgraded())
    }

    fn shutdown(&self) -> Result<()> {
        {
            let mut socket = self.writer.lock()?;
            if let Some(tls) = self.tls.get() {
                let _ = socket.write_all(&tls.close_notify()?);
            }

            // Also wakes up a blocked reader
            let _ = socket.shutdown(Shutdown::Both);
        }
        Ok(())
    }
}

/// Read a message header and its payload from `reader`, receiving data from `socket`.
fn read_header_and_payload(
    reader: &mut impl Read,
    socket: &TcpStream,
    read_timeout: Duration,
) -> Result<(ADBTransportMessageHeader, Vec<u8>)> {
    // Only wait `read_timeout` for the message to start: data is not consumed by a read that times out
    socket.set_read_timeout(Some(read_timeout))?;
    let mut data = [0; 24];
    let read = match reader.read(&mut data)? {
        0 => return Err(std::io::Error::from(ErrorKind::UnexpectedEof).into()),
        read => read,
    };

    socket.set_read_timeout(Some(MESSAGE_END_TIMEOUT))?;
    reader.read_exact(&mut data[read..])?;

    let header = ADBTransportMessageHeader::try_from(data)?;
    let mut msg_data = vec![0_u8; header.data_length() as usize];
    reader.read_exact(&mut msg_data)?;
    Ok((header, msg_data))
}

/// TLS state of a connection, shared by its reading and writing sides.
///
/// rustls' `StreamOwned` is not used: its reads block on the socket while holding the TLS state, which would
/// prevent writing while waiting for a message. Here, the state is only locked while data is encrypted or
/// decrypted.
#[derive(Debug)]
struct SharedTls(Mutex<ClientConnection>);

impl SharedTls {
    /// Do the TLS handshake on `socket`.
    fn handshake(mut tls: ClientConnection, socket: &mut TcpStream) -> Result<Self> {
        // Encrypted messages can be bigger than the default buffer limit
        tls.set_buffer_limit(None);

        socket.set_read_timeout(Some(MESSAGE_END_TIMEOUT))?;
        socket.set_write_timeout(Some(MESSAGE_END_TIMEOUT))?;
        while tls.is_handshaking() {
            tls.complete_io(socket)?;
        }

        Ok(Self(Mutex::new(tls)))
    }

    /// Encrypt `data`, and return the TLS records to send.
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(self.with_locked(|tls| {
            tls.writer().write_all(data)?;
            Self::pending_records(tls)
        })?)
    }

    /// Return the TLS records closing the connection.
    fn close_notify(&self) -> Result<Vec<u8>> {
        Ok(self.with_locked(|tls| {
            tls.send_close_notify();
            Self::pending_records(tls)
        })?)
    }

    fn pending_records(tls: &mut ClientConnection) -> std::io::Result<Vec<u8>> {
        let mut records = Vec::new();
        while tls.wants_write() {
            tls.write_tls(&mut records)?;
        }
        Ok(records)
    }

    /// Read decrypted data already received.
    fn read_decrypted(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.with_locked(|tls| tls.reader().read(buf))
    }

    /// Decrypt `received` data, making it available to [`Self::read_decrypted`].
    fn decrypt(&self, mut received: &[u8]) -> std::io::Result<()> {
        self.with_locked(|tls| {
            while !received.is_empty() {
                if tls.read_tls(&mut received)? == 0 {
                    return Err(std::io::Error::other("TLS buffer full"));
                }
                tls.process_new_packets().map_err(std::io::Error::other)?;
            }
            Ok(())
        })
    }

    /// Run `f` with the TLS state locked.
    fn with_locked<R>(
        &self,
        f: impl FnOnce(&mut ClientConnection) -> std::io::Result<R>,
    ) -> std::io::Result<R> {
        let mut tls = self
            .0
            .lock()
            .map_err(|_| std::io::Error::other("TLS state lock poisoned"))?;
        f(&mut tls)
    }

    /// Return a [`Read`] implementation decrypting the data received on `socket`.
    const fn reader<'a>(&'a self, socket: &'a TcpStream) -> TlsReader<'a> {
        TlsReader {
            tls: self,
            socket,
            received: Vec::new(),
        }
    }
}

/// [`Read`] implementation returning the data received on a socket, decrypted.
struct TlsReader<'a> {
    tls: &'a SharedTls,
    socket: &'a TcpStream,
    /// Buffer receiving encrypted data, allocated once needed.
    received: Vec<u8>,
}

impl Read for TlsReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            match self.tls.read_decrypted(buf) {
                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                result => return result,
            }

            // No decrypted data available: wait for more, without locking the TLS state
            if self.received.is_empty() {
                self.received = vec![0; MAX_TLS_RECORD_SIZE];
            }
            let size = self.socket.read(&mut self.received)?;
            if size == 0 {
                return Ok(0);
            }

            self.tls.decrypt(&self.received[..size])?;
        }
    }
}

/// Transport running on TCP
#[derive(Clone, Debug)]
pub struct TcpTransport {
    address: SocketAddr,
    current_connection: Option<Arc<Connection>>,
    private_key_path: PathBuf,
}

fn certificate_from_pk(key_pair: &KeyPair) -> Result<Vec<CertificateDer<'static>>> {
    let certificate_params = CertificateParams::default();
    let certificate = certificate_params.self_signed(key_pair)?;
    Ok(vec![certificate.der().to_owned()])
}

impl TcpTransport {
    /// Instantiate a new [`TcpTransport`] using a given private key
    pub fn new<A: Into<SocketAddr>, P: AsRef<Path>>(address: A, private_key_path: P) -> Self {
        Self {
            address: address.into(),
            current_connection: None,
            private_key_path: private_key_path.as_ref().to_path_buf(),
        }
    }

    fn get_current_connection(&self) -> Result<&Connection> {
        self.current_connection
            .as_deref()
            .ok_or(RustADBError::IOError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )))
    }
}

impl ADBTransport for TcpTransport {
    fn connect(&mut self) -> Result<()> {
        let stream = TcpStream::connect(self.address)?;
        self.current_connection = Some(Arc::new(Connection::new(stream)?));
        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        log::debug!("disconnecting...");
        if let Some(current_connection) = &self.current_connection {
            current_connection.shutdown()?;
        }

        Ok(())
    }
}

impl ADBMessageTransport for TcpTransport {
    fn read_message_with_timeout(&mut self, read_timeout: Duration) -> Result<ADBTransportMessage> {
        self.get_current_connection()?.read_message(read_timeout)
    }

    fn write_message_with_timeout(
        &mut self,
        message: ADBTransportMessage,
        write_timeout: Duration,
    ) -> Result<()> {
        self.get_current_connection()?
            .write_message(&message, write_timeout)
    }

    fn upgrade_connection(&mut self) -> Result<()> {
        let Some(current_connection) = &self.current_connection else {
            return Err(RustADBError::UpgradeError(
                "cannot upgrade a non-existing connection...".into(),
            ));
        };

        // TODO: Check if we cannot be more precise
        let pk_content = read_to_string(&self.private_key_path)?;

        let key_pair = KeyPair::from_pkcs8_pem_and_sign_algo(&pk_content, &PKCS_RSA_SHA256)?;

        let certificate = certificate_from_pk(&key_pair)?;
        let private_key = PrivatePkcs8KeyDer::from_pem_file(&self.private_key_path)?;

        let mut client_config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCertificateVerification {}))
            .with_client_auth_cert(certificate, private_key.into())?;

        client_config.key_log = Arc::new(KeyLogFile::new());

        let rc_config = Arc::new(client_config);
        let server_name = self.address.ip().into();
        let conn = ClientConnection::new(rc_config, server_name)?;

        current_connection.upgrade(conn)?;
        log::debug!("TLS handshake done");
        Ok(())
    }
}

#[derive(Debug)]
struct NoCertificateVerification;

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA1,
            SignatureScheme::ECDSA_SHA1_Legacy,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
        ]
    }
}
