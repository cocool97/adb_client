use std::fmt::Display;

pub enum SyncCommand {
    /// List files in a folder
    List,
    /// Receive a file from the device
    Recv,
    /// Send a file to the device
    Send,
    // Stat a file
    Stat,
}

impl Display for SyncCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncCommand::List => write!(f, "LIST"),
            SyncCommand::Recv => write!(f, "RECV"),
            SyncCommand::Send => write!(f, "SEND"),
            SyncCommand::Stat => write!(f, "STAT"),
        }
    }
}
