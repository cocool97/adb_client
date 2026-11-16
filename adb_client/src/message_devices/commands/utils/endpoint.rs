use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use crate::{Result, RustADBError};

/// Parse the port of a host `tcp:<port>` endpoint, the only kind of host endpoint supported.
pub fn parse_tcp_endpoint(endpoint: &str) -> Result<u16> {
    endpoint
        .strip_prefix("tcp:")
        .and_then(|port| port.parse().ok())
        .ok_or_else(|| {
            RustADBError::ADBRequestFailed(format!(
                "unsupported host endpoint '{endpoint}', expected tcp:<port>"
            ))
        })
}

/// IPv4 and IPv6 loopback addresses of `port`, in the order adb tries them.
pub fn loopback_addresses(port: u16) -> [SocketAddr; 2] {
    [
        (Ipv4Addr::LOCALHOST, port).into(),
        (Ipv6Addr::LOCALHOST, port).into(),
    ]
}
