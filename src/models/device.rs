use std::fmt::Display;

use crate::DeviceState;

pub struct Device {
    pub identifier: String,
    pub state: DeviceState,
}

impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.identifier, self.state)
    }
}
