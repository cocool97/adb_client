/// Represent a session between an ADBDevice and remote `adbd`.
#[derive(Debug, Clone, Copy)]
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

    pub fn local_id(self) -> u32 {
        self.local_id
    }

    pub fn remote_id(self) -> u32 {
        self.remote_id
    }
}
