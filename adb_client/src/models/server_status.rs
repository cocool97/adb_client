use std::fmt::Display;

use crate::RustADBError;

#[derive(Debug, Clone, PartialEq)]
pub enum UsbBackend {
    UNKNOWN,
    NATIVE,
    LIBUSB,
}

impl Display for UsbBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UsbBackend::UNKNOWN => write!(f, "UNKNOWN"),
            UsbBackend::NATIVE => write!(f, "NATIVE"),
            UsbBackend::LIBUSB => write!(f, "LIBUSB"),
        }
    }
}

impl From<i32> for UsbBackend {
    fn from(value: i32) -> UsbBackend {
        match value {
            1 => UsbBackend::NATIVE,
            2 => UsbBackend::LIBUSB,
            _ => UsbBackend::UNKNOWN,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MDNSBackend {
    UNKNOWN,
    BONJOUR,
    OPENSCREEN,
}

impl Display for MDNSBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MDNSBackend::UNKNOWN => write!(f, "UNKNOWN"),
            MDNSBackend::BONJOUR => write!(f, "BONJOUR"),
            MDNSBackend::OPENSCREEN => write!(f, "OPENSCREEN"),
        }
    }
}

impl From<i32> for MDNSBackend {
    fn from(value: i32) -> MDNSBackend {
        match value {
            1 => MDNSBackend::BONJOUR,
            2 => MDNSBackend::OPENSCREEN,
            _ => MDNSBackend::UNKNOWN,
        }
    }
}

/// Represents Server Status
#[derive(Debug, Clone)]
pub struct ServerStatus {
    /// usb backend
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
        write!(f, "usb_backend: {}\n", self.usb_backend)?;
        if self.usb_backend_forced {
            write!(f, "usb_backend_forced: {}\n", self.usb_backend_forced)?;
        }
        write!(f, "mdns_backend: {}\n", self.mdns_backend)?;
        if self.mdns_backend_forced {
            write!(f, "mdns_backend_forced: {}\n", self.mdns_backend_forced)?;
        }
        write!(f, "version: \"{}\"\n", self.version)?;
        write!(f, "build: \"{}\"\n", self.build)?;
        write!(f, "executable_absolute_path: \"{}\"\n", self.executable_absolute_path)?;
        write!(f, "log_absolute_path: \"{}\"\n", self.log_absolute_path)?;
        write!(f, "os: \"{}\"\n", self.os)
    }
}

fn parse_tag(cursor: &mut &[u8]) -> (u8, u8) {
    let tag = cursor[0];
    *cursor = &cursor[1..];
    let field_number = (tag >> 3) & 0x1F;
    let wire_type = tag & 0x07;
    (field_number, wire_type)
}

fn parse_varint(cursor: &mut &[u8]) -> i32 {
    let mut value = 0;
    let mut shift = 0;

    loop {
        let byte = cursor[0];
        *cursor = &cursor[1..];

        value |= (byte as i32 & 0x7F) << shift;

        if byte & 0x80 == 0 {
            break;
        }

        shift += 7;
    }

    value
}

fn parse_bool(cursor: &mut &[u8]) -> bool {
    parse_varint(cursor) != 0
}

fn parse_string(cursor: &mut &[u8]) -> Result<String, RustADBError> {
    let length = parse_varint(cursor) as usize;
    let str_bytes = &cursor[..length];
    *cursor = &cursor[length..];
    Ok(String::from_utf8(str_bytes.to_vec())?)
}

impl TryFrom<Vec<u8>> for ServerStatus {
    type Error = RustADBError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut server_status = ServerStatus {
            usb_backend: UsbBackend::UNKNOWN,
            usb_backend_forced: false,
            mdns_backend: MDNSBackend::UNKNOWN,
            mdns_backend_forced: false,
            version: "".to_string(),
            build: "".to_string(),
            executable_absolute_path: "".to_string(),
            log_absolute_path: "".to_string(),
            os: "".to_string(),
        };
        let mut cursor = &value[..];
        while !cursor.is_empty() {
            let (field_number, wire_type) = parse_tag(&mut cursor);
            match field_number {
                1 => {
                    if wire_type == 0 { // varint
                        let value = parse_varint(&mut cursor);
                        server_status.usb_backend = UsbBackend::from(value);
                    }
                }
                2 => {
                    if wire_type == 0 { // varint
                        let value = parse_bool(&mut cursor);
                        server_status.usb_backend_forced = value;
                    }
                }
                3 => {
                    if wire_type == 0 { // varint
                        let value = parse_varint(&mut cursor);
                        server_status.mdns_backend = MDNSBackend::from(value);
                    }
                }
                4 => {
                    if wire_type == 0 { // varint
                        let value = parse_bool(&mut cursor);
                        server_status.mdns_backend_forced = value;
                    }
                }
                5 => {
                    if wire_type == 2 { // length-delimited
                        let value = parse_string(&mut cursor)?;
                        server_status.version = value;
                    }
                }
                6 => {
                    if wire_type == 2 { // length-delimited
                        let value = parse_string(&mut cursor)?;
                        server_status.build = value;
                    }
                }
                7 => {
                    if wire_type == 2 { // length-delimited
                        let value = parse_string(&mut cursor)?;
                        server_status.executable_absolute_path = value;
                    }
                }
                8 => {
                    if wire_type == 2 { // length-delimited
                        let value = parse_string(&mut cursor)?;
                        server_status.log_absolute_path = value;
                    }
                }
                9 => {
                    if wire_type == 2 { // length-delimited
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