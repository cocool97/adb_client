use thiserror::Error;

/// Custom Result type thrown by this crate.
pub type Result<T> = std::result::Result<T, RustADBError>;

/// Represents all error types that can be thrown by the crate.
#[derive(Error, Debug)]
pub enum RustADBError {
    /// Indicates that an error occurred with I/O.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    /// Indicates that an error occurred when sending ADB request.
    #[error("ADB request failed - {0}")]
    ADBRequestFailed(String),
    /// Indicates that ADB server responded an unknown response type.
    #[error("Unknown response type {0}")]
    UnknownResponseType(String),
    /// Indicated that an unexpected command has been received
    #[error("Wrong response command received: {0}. Expected {1}")]
    WrongResponseReceived(String, String),
    /// Indicates that ADB server responses an unknown device state.
    #[error("Unknown device state {0}")]
    UnknownDeviceState(String),
    /// Indicates that an error occurred during UTF-8 parsing.
    #[error(transparent)]
    Utf8StrError(#[from] std::str::Utf8Error),
    /// Indicates that an error occurred during UTF-8 parsing.
    #[error(transparent)]
    Utf8StringError(#[from] std::string::FromUtf8Error),
    /// Indicates that the provided address is not a correct IP address.
    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),
    /// Indicates an error with regexps.
    #[error(transparent)]
    RegexError(#[from] regex::Error),
    /// Indicates that parsing regex did not worked.
    #[error("Regex parsing error: missing field")]
    RegexParsingError,
    /// Indicates an error with the integer conversion.
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    /// Indicates that an error occurred when converting a value.
    #[error("Conversion error")]
    ConversionError,
    /// Remote ADB server does not support shell feature.
    #[error("Remote ADB server does not support shell feature")]
    ADBShellNotSupported,
    /// Desired device has not been found
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    /// Indicates that the device must be paired before attempting a connection over WI-FI
    #[error("Device not paired before attempting to connect")]
    ADBDeviceNotPaired,
    /// An error occurred when getting device's framebuffer image
    #[error(transparent)]
    FramebufferImageError(#[from] image::error::ImageError),
    /// An error occurred when converting framebuffer content
    #[error("Cannot convert framebuffer into image")]
    FramebufferConversionError,
    /// Unimplemented framebuffer image version
    #[error("Unimplemented framebuffer image version: {0}")]
    UnimplementedFramebufferImageVersion(u32),
    /// An error occurred while getting user's home directory
    #[error(transparent)]
    HomeError(#[from] homedir::GetHomeError),
    /// Cannot get home directory
    #[error("Cannot get home directory")]
    NoHomeDirectory,
    /// Generic USB error
    #[error("USB Error: {0}")]
    UsbError(#[from] rusb::Error),
    /// USB device not found
    #[error("USB Device not found: {0} {1}")]
    USBDeviceNotFound(u16, u16),
    /// No descriptor found
    #[error("No USB descriptor found")]
    USBNoDescriptorFound,
    /// Integrity of the received message cannot be validated
    #[error("Invalid integrity. Expected CRC32 {0}, got {1}")]
    InvalidIntegrity(u32, u32),
    /// Error while decoding base64 data
    #[error(transparent)]
    Base64DecodeError(#[from] base64::DecodeError),
    /// Error while encoding base64 data
    #[error(transparent)]
    Base64EncodeError(#[from] base64::EncodeSliceError),
    /// An error occurred with RSA engine
    #[error(transparent)]
    RSAError(#[from] rsa::errors::Error),
    /// Cannot convert given data from slice
    #[error(transparent)]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    /// Given path does not represent an APK
    #[error("wrong file extension: {0}")]
    WrongFileExtension(String),
    /// An error occurred with PKCS8 data
    #[error("error with pkcs8: {0}")]
    RsaPkcs8Error(#[from] rsa::pkcs8::Error),
    /// Error during certificate generation
    #[error(transparent)]
    CertificateGenerationError(#[from] rcgen::Error),
    /// TLS Error
    #[error(transparent)]
    TLSError(#[from] rustls::Error),
    /// PEM certificate error
    #[error(transparent)]
    PemCertError(#[from] rustls_pki_types::pem::Error),
    /// Error while locking mutex
    #[error("error while locking data")]
    PoisonError,
    /// Cannot upgrade connection from TCP to TLS
    #[error("upgrade error: {0}")]
    UpgradeError(String),
    /// An error occurred while getting mdns devices
    #[error(transparent)]
    MDNSError(#[from] mdns_sd::Error),
    /// An error occurred while sending data to channel
    #[error(transparent)]
    SendError(#[from] std::sync::mpsc::SendError<crate::MDNSDevice>),
    /// An unknown transport has been provided
    #[error("unknown transport: {0}")]
    UnknownTransport(String),
}

impl<T> From<std::sync::PoisonError<T>> for RustADBError {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Self::PoisonError
    }
}
