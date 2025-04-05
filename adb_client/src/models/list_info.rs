#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
/// A list entry on the remote device
pub struct ADBListItem {
    /// The name of the file, not the path
    pub name: String,
    /// The unix time stamp of when it was last modified
    pub time: u32,
    /// The unix mode of the file, used for permissions and special bits
    pub permissions: u32,
    /// The size of the file
    pub size: u32,
    /// The type of item this is, file, directory or symlink
    pub item_type: ADBListItemType,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
/// The different types of item that the list item can be
pub enum ADBListItemType {
    /// The entry is a file
    File,
    /// The entry is a directory
    Directory,
    /// The entry is a symlink
    Symlink,
}
