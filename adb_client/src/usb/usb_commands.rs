use std::fmt::Display;

use crate::RustADBError;

#[derive(Debug, Eq, PartialEq)]
pub enum USBCommand {
    // Connect to a device
    Cnxn,
    // Close connection to a device
    Clse,
    // Device ask for authentication
    Auth, // OTHERS
          // A_SYNC 0x434e5953
          // A_OPEN 0x4e45504f
          // A_OKAY 0x59414b4f
          // A_WRTE 0x45545257
          // A_STLS 0x534C5453
}

impl USBCommand {
    pub fn to_u32(&self) -> u32 {
        match self {
            Self::Cnxn => 0x4e584e43,
            Self::Clse => 0x45534c43,
            Self::Auth => 0x48545541,
        }
    }
}

impl Display for USBCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            USBCommand::Cnxn => write!(f, "CNXN"),
            USBCommand::Clse => write!(f, "CLSE"),
            USBCommand::Auth => write!(f, "AUTH"),
        }
    }
}

impl TryFrom<&[u8]> for USBCommand {
    type Error = RustADBError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match u32::from_le_bytes(value.try_into().unwrap()) {
            0x4e584e43 => Ok(Self::Cnxn),
            0x45534c43 => Ok(Self::Clse),
            0x48545541 => Ok(Self::Auth),
            _ => Err(RustADBError::ConversionError),
        }
    }
}
