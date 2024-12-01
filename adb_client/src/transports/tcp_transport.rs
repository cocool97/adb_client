use rcgen::{CertificateParams, KeyPair, PKCS_RSA_SHA256};
use rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    pki_types::{pem::PemObject, CertificateDer, PrivatePkcs8KeyDer},
    ClientConfig, ClientConnection, KeyLogFile, SignatureScheme, StreamOwned,
};

use super::{ADBMessageTransport, ADBTransport};
use crate::{
    device::{
        get_default_adb_key_path, ADBTransportMessage, ADBTransportMessageHeader, MessageCommand,
    },
    Result, RustADBError,
};
use std::{
    fs::read_to_string,
    io::{Read, Write},
    net::{Shutdown, SocketAddr, TcpStream},
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Debug)]
enum CurrentConnection {
    Tcp(TcpStream),
    Tls(Box<StreamOwned<ClientConnection, TcpStream>>),
}

impl CurrentConnection {
    fn set_read_timeout(&self, read_timeout: Duration) -> Result<()> {
        match self {
            CurrentConnection::Tcp(tcp_stream) => {
                Ok(tcp_stream.set_read_timeout(Some(read_timeout))?)
            }
            CurrentConnection::Tls(stream_owned) => {
                Ok(stream_owned.sock.set_read_timeout(Some(read_timeout))?)
            }
        }
    }

    fn set_write_timeout(&self, write_timeout: Duration) -> Result<()> {
        match self {
            CurrentConnection::Tcp(tcp_stream) => {
                Ok(tcp_stream.set_write_timeout(Some(write_timeout))?)
            }
            CurrentConnection::Tls(stream_owned) => {
                Ok(stream_owned.sock.set_write_timeout(Some(write_timeout))?)
            }
        }
    }
}

impl Read for CurrentConnection {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            CurrentConnection::Tcp(tcp_stream) => tcp_stream.read(buf),
            CurrentConnection::Tls(tls_conn) => tls_conn.read(buf),
        }
    }
}

impl Write for CurrentConnection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            CurrentConnection::Tcp(tcp_stream) => tcp_stream.write(buf),
            CurrentConnection::Tls(tls_conn) => tls_conn.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            CurrentConnection::Tcp(tcp_stream) => tcp_stream.flush(),
            CurrentConnection::Tls(tls_conn) => tls_conn.flush(),
        }
    }
}

/// Transport running on USB
#[derive(Clone, Debug)]
pub struct TcpTransport {
    address: SocketAddr,
    current_connection: Option<Arc<Mutex<CurrentConnection>>>,
    private_key_path: PathBuf,
}

fn certificate_from_pk(key_pair: &KeyPair) -> Result<Vec<CertificateDer<'static>>> {
    let certificate_params = CertificateParams::default();
    let certificate = certificate_params.self_signed(key_pair)?;
    Ok(vec![certificate.der().to_owned()])
}

impl TcpTransport {
    /// Instantiate a new [`TcpTransport`]
    pub fn new(address: SocketAddr) -> Result<Self> {
        Self::new_with_custom_private_key(address, get_default_adb_key_path()?)
    }

    /// Instantiate a new [`TcpTransport`] using a given private key
    pub fn new_with_custom_private_key(
        address: SocketAddr,
        private_key_path: PathBuf,
    ) -> Result<Self> {
        Ok(Self {
            address,
            current_connection: None,
            private_key_path,
        })
    }

    fn get_current_connection(&mut self) -> Result<Arc<Mutex<CurrentConnection>>> {
        self.current_connection
            .as_ref()
            .ok_or(RustADBError::IOError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )))
            .cloned()
    }

    pub(crate) fn upgrade_connection(&mut self) -> Result<()> {
        let current_connection = match self.current_connection.clone() {
            Some(current_connection) => current_connection,
            None => {
                return Err(RustADBError::UpgradeError(
                    "cannot upgrade a non-existing connection...".into(),
                ))
            }
        };

        {
            let mut current_conn_locked = current_connection.lock()?;
            match current_conn_locked.deref() {
                CurrentConnection::Tcp(tcp_stream) => {
                    // TODO: Check if we cannot be more precise

                    let pk_content = read_to_string(&self.private_key_path)?;

                    let key_pair =
                        KeyPair::from_pkcs8_pem_and_sign_algo(&pk_content, &PKCS_RSA_SHA256)?;

                    let certificate = certificate_from_pk(&key_pair)?;
                    let private_key = PrivatePkcs8KeyDer::from_pem_file(&self.private_key_path)?;

                    let mut client_config = ClientConfig::builder()
                        .dangerous()
                        .with_custom_certificate_verifier(Arc::new(NoCertificateVerification {}))
                        .with_client_auth_cert(certificate, private_key.into())?;

                    client_config.key_log = Arc::new(KeyLogFile::new());

                    let rc_config = Arc::new(client_config);
                    let example_com = self.address.ip().into();
                    let conn = ClientConnection::new(rc_config, example_com)?;
                    let owned = tcp_stream.try_clone()?;
                    let client = StreamOwned::new(conn, owned);

                    // Update current connection state to now use TLS protocol
                    *current_conn_locked = CurrentConnection::Tls(Box::new(client));
                }
                CurrentConnection::Tls(_) => {
                    return Err(RustADBError::UpgradeError(
                        "cannot upgrade a TLS connection...".into(),
                    ))
                }
            }
        }

        let message = self.read_message()?;
        match message.header().command() {
            MessageCommand::Cnxn => {
                let device_infos = String::from_utf8(message.into_payload())?;
                log::debug!("received device info: {device_infos}");
                Ok(())
            }
            c => Err(RustADBError::ADBRequestFailed(format!(
                "Wrong command received {}",
                c
            ))),
        }
    }
}

impl ADBTransport for TcpTransport {
    fn connect(&mut self) -> Result<()> {
        let stream = TcpStream::connect(self.address)?;
        self.current_connection = Some(Arc::new(Mutex::new(CurrentConnection::Tcp(stream))));
        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        log::debug!("disconnecting...");
        if let Some(current_connection) = &self.current_connection {
            let mut lock = current_connection.lock()?;
            match lock.deref_mut() {
                CurrentConnection::Tcp(tcp_stream) => {
                    let _ = tcp_stream.shutdown(Shutdown::Both);
                }
                CurrentConnection::Tls(tls_conn) => {
                    tls_conn.conn.send_close_notify();
                    let _ = tls_conn.sock.shutdown(Shutdown::Both);
                }
            }
        }

        Ok(())
    }
}

impl ADBMessageTransport for TcpTransport {
    fn read_message_with_timeout(
        &mut self,
        read_timeout: std::time::Duration,
    ) -> Result<crate::device::ADBTransportMessage> {
        let raw_connection_lock = self.get_current_connection()?;
        let mut raw_connection = raw_connection_lock.lock()?;

        raw_connection.set_read_timeout(read_timeout)?;

        let mut data = [0; 24];
        let mut total_read = 0;
        loop {
            total_read += raw_connection.read(&mut data[total_read..])?;
            if total_read == data.len() {
                break;
            }
        }

        let header = ADBTransportMessageHeader::try_from(data)?;

        if header.data_length() != 0 {
            let mut msg_data = vec![0_u8; header.data_length() as usize];
            let mut total_read = 0;
            loop {
                total_read += raw_connection.read(&mut msg_data[total_read..])?;
                if total_read == msg_data.capacity() {
                    break;
                }
            }

            let message = ADBTransportMessage::from_header_and_payload(header, msg_data);

            // Check message integrity
            if !message.check_message_integrity() {
                return Err(RustADBError::InvalidIntegrity(
                    ADBTransportMessageHeader::compute_crc32(message.payload()),
                    message.header().data_crc32(),
                ));
            }

            return Ok(message);
        }

        Ok(ADBTransportMessage::from_header_and_payload(header, vec![]))
    }

    fn write_message_with_timeout(
        &mut self,
        message: ADBTransportMessage,
        write_timeout: Duration,
    ) -> Result<()> {
        let message_bytes = message.header().as_bytes()?;
        let raw_connection_lock = self.get_current_connection()?;
        let mut raw_connection = raw_connection_lock.lock()?;

        raw_connection.set_write_timeout(write_timeout)?;

        let mut total_written = 0;
        loop {
            total_written += raw_connection.write(&message_bytes[total_written..])?;
            if total_written == message_bytes.len() {
                raw_connection.flush()?;
                break;
            }
        }

        let payload = message.into_payload();
        if !payload.is_empty() {
            let mut total_written = 0;
            loop {
                total_written += raw_connection.write(&payload[total_written..])?;
                if total_written == payload.len() {
                    raw_connection.flush()?;
                    break;
                }
            }
        }

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
