use std::{
    collections::HashSet,
    fmt::Display,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use mdns_sd::{ResolvedService, ScopedIp};

/// Represent a device found from mdns search
#[derive(Debug)]
pub struct MDNSDevice {
    /// Full device address when resolved
    pub fullname: String,
    /// Device IP addresses
    addresses: HashSet<IpAddr>,
}

impl MDNSDevice {
    /// Return all adresses linked to this device
    #[must_use]
    pub fn addresses(&self) -> HashSet<IpAddr> {
        self.addresses.clone()
    }

    /// Return all IPv4 addresses linked to this device
    #[must_use]
    pub fn ipv4_addresses(&self) -> HashSet<Ipv4Addr> {
        self.addresses
            .iter()
            .filter_map(|addr| match addr {
                IpAddr::V4(addr) => Some(addr),
                IpAddr::V6(_) => None,
            })
            .copied()
            .collect()
    }

    /// Return all IPv6 addresses linked to this device
    #[must_use]
    pub fn ipv6_addresses(&self) -> HashSet<Ipv6Addr> {
        self.addresses
            .iter()
            .filter_map(|addr| match addr {
                IpAddr::V4(_) => None,
                IpAddr::V6(addr) => Some(addr),
            })
            .copied()
            .collect()
    }
}

impl From<Box<ResolvedService>> for MDNSDevice {
    fn from(value: Box<ResolvedService>) -> Self {
        Self {
            fullname: value.fullname,
            addresses: value.addresses.iter().map(ScopedIp::to_ip_addr).collect(),
        }
    }
}

impl Display for MDNSDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Device fullname: {}", self.fullname)?;
        writeln!(f, "IPv4 Addresses: {:?}", self.ipv4_addresses())?;
        write!(f, "IPv6 Addresses: {:?}", self.ipv6_addresses())?;

        Ok(())
    }
}
