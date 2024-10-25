use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum USBCommand {
    /// Connect to a device
    Cnxn = 0x4e584e43,
    /// Close connection to a device
    Clse = 0x45534c43,
    /// Device ask for authentication
    Auth = 0x48545541,
    /// Open a data connection
    Open = 0x4e45504f,
    /// Write data to connection
    Write = 0x45545257,
    /// Server understood the message
    Okay = 0x59414b4f,
    // Sync 0x434e5953
    // Stls 0x534C5453
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum USBSubcommand {
    Stat = 0x54415453,
    Send = 0x444E4553,
    Recv = 0x56434552,
    Quit = 0x54495551,
    Fail = 0x4c494146,
    Done = 0x454e4f44,
    Data = 0x41544144,
    List = 0x5453494c,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubcommandWithArg {
    subcommand: USBSubcommand,
    arg: u32,
}

impl USBSubcommand {
    pub fn with_arg(self, arg: u32) -> SubcommandWithArg {
        SubcommandWithArg {
            subcommand: self,
            arg,
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
