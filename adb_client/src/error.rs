use thiserror::Error;

/// Custom Result type thrown by this crate.
pub type Result<T> = std::result::Result<T, RustADBError>;

/// Represents all error types that can be thrown by the crate.
#[derive(Error, Debug)]
pub enum RustADBError {
    /// Indicates that an error occured with I/O.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    /// Indicates that an error occured when sending ADB request.
    #[error("ADB request failed - {0}")]
    ADBRequestFailed(String),
    /// Indicates that ADB server responded an unknown response type.
    #[error("Unknown response type {0}")]
    UnknownResponseType(String),
    /// Indicates that ADB server responses an unknown device state.
    #[error("Unknown device state {0}")]
    UnknownDeviceState(String),
    /// Indicates that an error occured during UTF-8 parsing.
    #[error(transparent)]
    Utf8StrError(#[from] std::str::Utf8Error),
    /// Indicates that an error occured during UTF-8 parsing.
    #[error(transparent)]
    Utf8StringError(#[from] std::string::FromUtf8Error),
    /// Indicates that the provided address is not a correct IP address.
    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),
    /// Indicates an error with regexes.
    #[error(transparent)]
    RegexError(#[from] regex::Error),
    /// Indicates that parsing regex did not worked.
    #[error("Regex parsing error: missing field")]
    RegexParsingError,
    /// Indicates an error with the integer convertion.
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    /// Indicates that an error occured when converting a value.
    #[error("Convertion error")]
    ConvertionError,
    /// Remote ADB server does not support shell feature.
    #[error("Remote ADB server does not support shell feature")]
    ADBShellNotSupported,
}
