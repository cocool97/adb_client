use std::fmt::Display;

/// The different types of item that the `ADBListItem` can represent.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ADBListItemType {
    /// The entry is a file
    File(ADBListItem),
    /// The entry is a directory
    Directory(ADBListItem),
    /// The entry is a symlink
    Symlink(ADBListItem),
}

impl Display for ADBListItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(item) => write!(f, "file: {item}"),
            Self::Directory(item) => write!(f, "directory: {item}"),
            Self::Symlink(item) => write!(f, "symlink: {item}"),
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
