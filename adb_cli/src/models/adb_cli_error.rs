use std::fmt::Display;

use adb_client::RustADBError;

pub type ADBCliResult<T> = Result<T, ADBCliError>;

pub enum ADBCliError {
    Standard(Box<dyn std::error::Error>),
    MayNeedAnIssue(Box<dyn std::error::Error>),
}

impl Display for ADBCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ADBCliError::Standard(error) => write!(f, "{error}"),
            ADBCliError::MayNeedAnIssue(error) => {
                write!(
                    f,
                    r"Error: {error}
                    An unexpected error occurred and may indicate a bug.
                    Please report this issue on the project repository (including steps to reproduce if possible): https://github.com/cocool97/adb_client/issues.",
                )
            }
        }
    }
}

impl From<std::io::Error> for ADBCliError {
    fn from(value: std::io::Error) -> Self {
        // We do not consider adb_cli related `std::io::error` as critical
        Self::Standard(Box::new(value))
    }
}

impl From<adb_client::RustADBError> for ADBCliError {
    fn from(value: adb_client::RustADBError) -> Self {
        let value = Box::new(value);

        match value.as_ref() {
            RustADBError::RegexParsingError
            | RustADBError::WrongResponseReceived(_, _)
            | RustADBError::FramebufferImageError(_)
            | RustADBError::FramebufferConversionError
            | RustADBError::UnimplementedFramebufferImageVersion(_)
            | RustADBError::IOError(_)
            | RustADBError::ADBRequestFailed(_)
            | RustADBError::UnknownDeviceState(_)
            | RustADBError::Utf8StrError(_)
            | RustADBError::Utf8StringError(_)
            | RustADBError::RegexError(_)
            | RustADBError::ParseIntError(_)
            | RustADBError::ConversionError
            | RustADBError::IntegerConversionError(_)
            | RustADBError::NoHomeDirectory
            | RustADBError::UsbError(_)
            | RustADBError::InvalidIntegrity(_, _)
            | RustADBError::Base64DecodeError(_)
            | RustADBError::Base64EncodeError(_)
            | RustADBError::RSAError(_)
            | RustADBError::TryFromSliceError(_)
            | RustADBError::RsaPkcs8Error(_)
            | RustADBError::CertificateGenerationError(_)
            | RustADBError::TLSError(_)
            | RustADBError::PemCertError(_)
            | RustADBError::PoisonError
            | RustADBError::UpgradeError(_)
            | RustADBError::MDNSError(_)
            | RustADBError::SendError
            | RustADBError::UnknownFileMode(_)
            | RustADBError::UnknownTransport(_)
            | RustADBError::RemountError(_) => Self::MayNeedAnIssue(value),
            // List of [`RustADBError`] that may occur in standard contexts and therefore do not require for issues
            RustADBError::ADBDeviceNotPaired
            | RustADBError::UnknownResponseType(_)
            | RustADBError::DeviceNotFound(_)
            | RustADBError::USBNoDescriptorFound
            | RustADBError::ADBShellNotSupported
            | RustADBError::USBDeviceNotFound(_, _)
            | RustADBError::WrongFileExtension(_)
            | RustADBError::AddrParseError(_)
            | RustADBError::DeviceBusy => Self::Standard(value),
        }
    }
}
