use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum MessageCommand {
    /// Connect to a device
    Cnxn = 0x4E584E43,
    /// Close connection to a device
    Clse = 0x45534C43,
    /// Device ask for authentication
    Auth = 0x48545541,
    /// Open a data connection
    Open = 0x4E45504F,
    /// Write data to connection
    Write = 0x45545257,
    /// Server understood the message
    Okay = 0x59414B4F,
    /// Start a connection using TLS
    Stls = 0x534C5453,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum MessageSubcommand {
    Stat = 0x54415453,
    Send = 0x444E4553,
    Recv = 0x56434552,
    Quit = 0x54495551,
    Fail = 0x4C494146,
    Done = 0x454E4F44,
    Data = 0x41544144,
    List = 0x5453494C,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubcommandWithArg {
    subcommand: MessageSubcommand,
    arg: u32,
}

impl MessageSubcommand {
    pub fn with_arg(self, arg: u32) -> SubcommandWithArg {
        SubcommandWithArg {
            subcommand: self,
            arg,
        }
    }
}

impl Display for MessageCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageCommand::Cnxn => write!(f, "CNXN"),
            MessageCommand::Clse => write!(f, "CLSE"),
            MessageCommand::Auth => write!(f, "AUTH"),
            MessageCommand::Open => write!(f, "OPEN"),
            MessageCommand::Write => write!(f, "WRTE"),
            MessageCommand::Okay => write!(f, "OKAY"),
            MessageCommand::Stls => write!(f, "STLS"),
        }
    }
}
