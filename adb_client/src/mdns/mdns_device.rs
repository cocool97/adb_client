use std::{collections::HashSet, net::IpAddr};

use mdns_sd::{ResolvedService, ScopedIp};

/// Represent a device found from mdns search
#[derive(Debug)]
pub struct MDNSDevice {
    /// Full device address when resolved
    pub fullname: String,
    /// Device IP addresses
    pub addresses: HashSet<IpAddr>,
}

impl From<Box<ResolvedService>> for MDNSDevice {
    fn from(value: Box<ResolvedService>) -> Self {
        Self {
            fullname: value.fullname,
            addresses: value.addresses.iter().map(ScopedIp::to_ip_addr).collect(),
        }
    }
}
