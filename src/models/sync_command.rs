pub enum SyncCommand<'a> {
    /// List files in a folder
    List(&'a str),
    /// Receive a file from the device
    Recv(&'a str, &'a mut dyn std::io::Write),
    /// Send a file to the device
    Send(&'a mut dyn std::io::Read, &'a str),
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
