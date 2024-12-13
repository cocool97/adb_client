use quick_protobuf::{BytesReader, MessageRead};

use std::fmt::Display;

use crate::RustADBError;

#[derive(Debug, PartialEq, Default, Eq, Clone, Copy)]
pub enum UsbBackend {
    #[default]
    Unknown = 0,
    Native = 1,
    LibUSB = 2,
}

impl From<i32> for UsbBackend {
    fn from(i: i32) -> Self {
        match i {
            0 => UsbBackend::Unknown,
            1 => UsbBackend::Native,
            2 => UsbBackend::LibUSB,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for UsbBackend {
    fn from(s: &'a str) -> Self {
        match s {
            "UNKNOWN_USB" => UsbBackend::Unknown,
            "NATIVE" => UsbBackend::Native,
            "LIBUSB" => UsbBackend::LibUSB,
            _ => Self::default(),
        }
    }
}

impl Display for UsbBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UsbBackend::Unknown => write!(f, "UNKNOWN_USB"),
            UsbBackend::Native => write!(f, "NATIVE"),
            UsbBackend::LibUSB => write!(f, "LIBUSB"),
        }
    }
}

/// MDNS Backend Status
#[derive(Debug, Clone, PartialEq, Default)]
pub enum MDNSBackend {
    #[default]
    /// Unknown
    Unknown = 0,
    /// Bonjour
    Bonjour = 1,
    /// OpenScreen
    OpenScreen = 2,
}

impl From<i32> for MDNSBackend {
    fn from(i: i32) -> Self {
        match i {
            0 => MDNSBackend::Unknown,
            1 => MDNSBackend::Bonjour,
            2 => MDNSBackend::OpenScreen,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for MDNSBackend {
    fn from(s: &'a str) -> Self {
        match s {
            "UNKNOWN_MDNS" => MDNSBackend::Unknown,
            "BONJOUR" => MDNSBackend::Bonjour,
            "OPENSCREEN" => MDNSBackend::OpenScreen,
            _ => Self::default(),
        }
    }
}

impl Display for MDNSBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MDNSBackend::Unknown => write!(f, "UNKNOWN_MDNS"),
            MDNSBackend::Bonjour => write!(f, "BONJOUR"),
            MDNSBackend::OpenScreen => write!(f, "OPENSCREEN"),
        }
    }
}

/// Structure representing current server status
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ServerStatus {
    /// Currently active USB backend
    pub usb_backend: UsbBackend,
    /// Is USB backend forced ?
    pub usb_backend_forced: bool,
    /// Currently active MDNS backend
    pub mdns_backend: MDNSBackend,
    /// Is MDNS backend forced ?
    pub mdns_backend_forced: bool,
    /// Server version
    pub version: String,
    /// Server build information
    pub build: String,
    /// Server executable absolute path
    pub executable_absolute_path: String,
    /// Server logs absolute path
    pub log_absolute_path: String,
    /// OS server is running on
    pub os: String,
}

impl<'a> MessageRead<'a> for ServerStatus {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> quick_protobuf::Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.usb_backend = r.read_enum(bytes)?,
                Ok(16) => msg.usb_backend_forced = r.read_bool(bytes)?,
                Ok(24) => msg.mdns_backend = r.read_enum(bytes)?,
                Ok(32) => msg.mdns_backend_forced = r.read_bool(bytes)?,
                Ok(42) => msg.version = r.read_string(bytes)?.to_string(),
                Ok(50) => msg.build = r.read_string(bytes)?.to_string(),
                Ok(58) => msg.executable_absolute_path = r.read_string(bytes)?.to_string(),
                Ok(66) => msg.log_absolute_path = r.read_string(bytes)?.to_string(),
                Ok(74) => msg.os = r.read_string(bytes)?.to_string(),
                Ok(t) => {
                    r.read_unknown(bytes, t)?;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl Display for ServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "usb_backend: {}", self.usb_backend)?;
        if self.usb_backend_forced {
            writeln!(f, "usb_backend_forced: {}", self.usb_backend_forced)?;
        }
        writeln!(f, "mdns_backend: {}", self.mdns_backend)?;
        if self.mdns_backend_forced {
            writeln!(f, "mdns_backend_forced: {}", self.mdns_backend_forced)?;
        }
        writeln!(f, "version: \"{}\"", self.version)?;
        writeln!(f, "build: \"{}\"", self.build)?;
        writeln!(
            f,
            "executable_absolute_path: \"{}\"",
            self.executable_absolute_path
        )?;
        writeln!(f, "log_absolute_path: \"{}\"", self.log_absolute_path)?;
        writeln!(f, "os: \"{}\"", self.os)
    }
}

impl TryFrom<Vec<u8>> for ServerStatus {
    type Error = RustADBError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut reader = BytesReader::from_bytes(&value);
        ServerStatus::from_reader(&mut reader, &value).map_err(|_| RustADBError::ConversionError)
    }
}
