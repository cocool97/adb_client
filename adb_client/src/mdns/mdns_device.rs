use std::{collections::HashSet, net::IpAddr};

/// Represent a device found from mdns search
#[derive(Debug)]
pub struct MDNSDevice {
    /// Full device address when resolved
    pub fullname: String,
    /// Device IP addresses
    pub addresses: HashSet<IpAddr>,
}

impl From<mdns_sd::ServiceInfo> for MDNSDevice {
    fn from(value: mdns_sd::ServiceInfo) -> Self {
        Self {
            fullname: value.get_fullname().to_string(),
            addresses: value.get_addresses().to_owned(),
        }
    }
}
