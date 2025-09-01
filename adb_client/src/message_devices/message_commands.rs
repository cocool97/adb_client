use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum MessageCommand {
    /// Connect to a device
    Cnxn = 0x4E58_4E43,
    /// Close connection to a device
    Clse = 0x4553_4C43,
    /// Device ask for authentication
    Auth = 0x4854_5541,
    /// Open a data connection
    Open = 0x4E45_504F,
    /// Write data to connection
    Write = 0x4554_5257,
    /// Server understood the message
    Okay = 0x5941_4B4F,
    /// Start a connection using TLS
    Stls = 0x534C_5453,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum MessageSubcommand {
    Stat = 0x5441_5453,
    Send = 0x444E_4553,
    Recv = 0x5643_4552,
    Quit = 0x5449_5551,
    Fail = 0x4C49_4146,
    Done = 0x454E_4F44,
    Data = 0x4154_4144,
    List = 0x5453_494C,
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
