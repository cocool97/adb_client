use std::fmt::Display;

/// The different types of item that the `ADBListItem` can represent.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ADBListItemType {
    /// The entry is a FIFO/named pipe
    Fifo(ADBListItem),
    /// The entry is a character device
    CharacterDevice(ADBListItem),
    /// The entry is a directory
    Directory(ADBListItem),
    /// The entry is a block device
    BlockDevice(ADBListItem),
    /// The entry is a file
    File(ADBListItem),
    /// The entry is a symlink
    Symlink(ADBListItem),
    /// The entry is a socket
    Socket(ADBListItem),
    /// The entry is some other type
    Other(ADBListItem),
}

impl Display for ADBListItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fifo(item) => write!(f, "fifo: {item}"),
            Self::CharacterDevice(item) => write!(f, "character device: {item}"),
            Self::Directory(item) => write!(f, "directory: {item}"),
            Self::BlockDevice(item) => write!(f, "block device: {item}"),
            Self::File(item) => write!(f, "file: {item}"),
            Self::Symlink(item) => write!(f, "symlink: {item}"),
            Self::Socket(item) => write!(f, "socket: {item}"),
            Self::Other(item) => write!(f, "other: {item}"),
        }
    }
}

/// An item list entry on the device.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ADBListItem {
    /// The name of the file, not the path
    pub name: String,
    /// The unix time stamp of when it was last modified
    pub time: u32,
    /// The unix mode of the file, used for permissions and special bits
    pub permissions: u32,
    /// The size of the file
    pub size: u32,
}

impl Display for ADBListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "name: {}, time: {}, size: {}, permissions: {:#o}",
            self.name, self.time, self.size, self.permissions
        )
    }
}
