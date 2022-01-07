use std::fmt::Display;
use std::str;

use crate::RustADBError;

pub struct AdbVersion {
    major: u32,
    minor: u32,
    revision: u32,
}

impl AdbVersion {
    pub fn new(minor: u32, revision: u32) -> Self {
        Self {
            major: 1,
            minor,
            revision,
        }
    }
}

impl Display for AdbVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.revision)
    }
}

impl TryFrom<Vec<u8>> for AdbVersion {
    type Error = RustADBError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(AdbVersion::new(
            u32::from_str_radix(str::from_utf8(&value[0..2])?, 16)?,
            u32::from_str_radix(str::from_utf8(&value[2..4])?, 16)?,
        ))
    }
}
