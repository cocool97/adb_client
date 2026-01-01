/// Represent a session between an `ADBDevice` and remote `adbd`.
#[derive(Debug)]
pub(crate) struct ADBSession {
    local_id: u32,
    remote_id: u32,
}

impl ADBSession {
    pub fn new(local_id: u32, remote_id: u32) -> Self {
        Self {
            local_id,
            remote_id,
        }
    }

    pub const fn local_id(self) -> u32 {
        self.local_id
    }

    pub const fn remote_id(self) -> u32 {
        self.remote_id
    }
}
