use thiserror::Error;

pub type Result<T> = std::result::Result<T, RustADBError>;

#[derive(Error, Debug)]
pub enum RustADBError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("ADB request failed")]
    ADBRequestFailed,
    #[error("Unknown result type {0}")]
    UnknownResultType(String),
    #[error("Unknown device state {0}")]
    UnknownDeviceState(String),
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),
}
