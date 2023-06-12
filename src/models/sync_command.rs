pub enum SyncCommand<'a> {
    /// List files in a folder
    List(&'a str),
    /// Retrieve a file from the device
    /// TODO: Instead of having a String, use a IO trait
    Recv(&'a str, String),
    /// Send a file to the device
    /// TODO: Instead of having a String, use a IO trait
    Send(&'a str, String),
    // Stat a file
    Stat(&'a str),
}

impl ToString for SyncCommand<'_> {
    fn to_string(&self) -> String {
        match self {
            SyncCommand::List(_) => "LIST",
            SyncCommand::Recv(_, _) => "RECV",
            SyncCommand::Send(_, _) => "SEND",
            SyncCommand::Stat(_) => "STAT",
        }
        .to_string()
    }
}
