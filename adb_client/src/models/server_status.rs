use std::fmt::Display;

use crate::RustADBError;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum UsbBackend {
    #[default]
    Unknown,
    Native,
    LibUSB,
}

impl Display for UsbBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UsbBackend::Unknown => write!(f, "UNKNOWN"),
            UsbBackend::Native => write!(f, "NATIVE"),
            UsbBackend::LibUSB => write!(f, "LIBUSB"),
        }
    }
}

impl From<i32> for UsbBackend {
    fn from(value: i32) -> UsbBackend {
        match value {
            1 => UsbBackend::Native,
            2 => UsbBackend::LibUSB,
            _ => UsbBackend::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum MDNSBackend {
    #[default]
    Unknown,
    Bonjour,
    OpenScreen,
}

impl Display for MDNSBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MDNSBackend::Unknown => write!(f, "UNKNOWN"),
            MDNSBackend::Bonjour => write!(f, "BONJOUR"),
            MDNSBackend::OpenScreen => write!(f, "OPENSCREEN"),
        }
    }
}

impl From<i32> for MDNSBackend {
    fn from(value: i32) -> MDNSBackend {
        match value {
            1 => MDNSBackend::Bonjour,
            2 => MDNSBackend::OpenScreen,
            _ => MDNSBackend::Unknown,
        }
    }
}

/// Structure representing current server status
#[derive(Debug, Clone, Default)]
pub struct ServerStatus {
    pub usb_backend: UsbBackend,
    pub usb_backend_forced: bool,
    pub mdns_backend: MDNSBackend,
    pub mdns_backend_forced: bool,
    pub version: String,
    pub build: String,
    pub executable_absolute_path: String,
    pub log_absolute_path: String,
    pub os: String,
}

impl Display for ServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "usb_backend: {}", self.usb_backend)?;
        writeln!(f, "usb_backend_forced: {}", self.usb_backend_forced)?;
        writeln!(f, "mdns_backend: {}", self.mdns_backend)?;
        writeln!(f, "mdns_backend_forced: {}", self.mdns_backend_forced)?;
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

fn parse_tag(cursor: &mut &[u8]) -> Result<(u8, u8), RustADBError> {
    if cursor.is_empty() {
        return Err(RustADBError::ConversionError);
    }
    let tag = cursor[0];
    *cursor = &cursor[1..];
    let field_number = (tag >> 3) & 0x1F;
    let wire_type = tag & 0x07;
    Ok((field_number, wire_type))
}

fn parse_varint(cursor: &mut &[u8]) -> Result<i32, RustADBError> {
    let mut value = 0;
    let mut shift = 0;

    loop {
        if cursor.is_empty() {
            return Err(RustADBError::ConversionError);
        }

        let byte = cursor[0];
        *cursor = &cursor[1..];

        value |= (byte as i32 & 0x7F) << shift;

        if byte & 0x80 == 0 {
            break;
        }

        shift += 7;
    }

    Ok(value)
}

fn parse_bool(cursor: &mut &[u8]) -> Result<bool, RustADBError> {
    Ok(parse_varint(cursor)? != 0)
}

fn parse_string(cursor: &mut &[u8]) -> Result<String, RustADBError> {
    let length = parse_varint(cursor)? as usize;
    let str_bytes = &cursor[..length];
    *cursor = &cursor[length..];
    Ok(String::from_utf8(str_bytes.to_vec())?)
}

impl TryFrom<Vec<u8>> for ServerStatus {
    type Error = RustADBError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut server_status = ServerStatus::default();
        let mut cursor = &value[..];
        while !cursor.is_empty() {
            let (field_number, wire_type) = parse_tag(&mut cursor)?;
            match field_number {
                1 => {
                    if wire_type == 0 {
                        let value = parse_varint(&mut cursor)?;
                        server_status.usb_backend = UsbBackend::from(value);
                    }
                }
                2 => {
                    if wire_type == 0 {
                        let value = parse_bool(&mut cursor)?;
                        server_status.usb_backend_forced = value;
                    }
                }
                3 => {
                    if wire_type == 0 {
                        let value = parse_varint(&mut cursor)?;
                        server_status.mdns_backend = MDNSBackend::from(value);
                    }
                }
                4 => {
                    if wire_type == 0 {
                        let value = parse_bool(&mut cursor)?;
                        server_status.mdns_backend_forced = value;
                    }
                }
                5 => {
                    if wire_type == 2 {
                        let value = parse_string(&mut cursor)?;
                        server_status.version = value;
                    }
                }
                6 => {
                    if wire_type == 2 {
                        let value = parse_string(&mut cursor)?;
                        server_status.build = value;
                    }
                }
                7 => {
                    if wire_type == 2 {
                        let value = parse_string(&mut cursor)?;
                        server_status.executable_absolute_path = value;
                    }
                }
                8 => {
                    if wire_type == 2 {
                        let value = parse_string(&mut cursor)?;
                        server_status.log_absolute_path = value;
                    }
                }
                9 => {
                    if wire_type == 2 {
                        let value = parse_string(&mut cursor)?;
                        server_status.os = value;
                    }
                }
                _ => {}
            }
        }
        Ok(server_status)
    }
}
