use std::fmt::Display;

use crate::RustADBError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum USBCommand {
    /// Connect to a device
    Cnxn,
    /// Close connection to a device
    Clse,
    /// Device ask for authentication
    Auth,
    /// Open a data connection
    Open,
    /// Server understood the message
    Okay,
    /// Write data to connection
    Write,
    // Sync 0x434e5953
    // Stls 0x534C5453
}

impl USBCommand {
    pub fn u32_value(&self) -> u32 {
        match self {
            Self::Cnxn => 0x4e584e43,
            Self::Clse => 0x45534c43,
            Self::Auth => 0x48545541,
            Self::Open => 0x4e45504f,
            Self::Write => 0x45545257,
            Self::Okay => 0x59414b4f,
        }
    }
}

impl Display for USBCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            USBCommand::Cnxn => write!(f, "CNXN"),
            USBCommand::Clse => write!(f, "CLSE"),
            USBCommand::Auth => write!(f, "AUTH"),
            USBCommand::Open => write!(f, "OPEN"),
            USBCommand::Write => write!(f, "WRTE"),
            USBCommand::Okay => write!(f, "OKAY"),
        }
    }
}

impl TryFrom<&[u8]> for USBCommand {
    type Error = RustADBError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match u32::from_le_bytes(value.try_into()?) {
            0x4e584e43 => Ok(Self::Cnxn),
            0x45534c43 => Ok(Self::Clse),
            0x48545541 => Ok(Self::Auth),
            0x4e45504f => Ok(Self::Open),
            0x45545257 => Ok(Self::Write),
            0x59414b4f => Ok(Self::Okay),
            _ => Err(RustADBError::ConversionError),
        }
    }
}
