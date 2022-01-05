use std::fmt::Display;

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
