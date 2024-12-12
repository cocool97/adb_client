use core::str;
use std::{fmt::Display, str::FromStr};

use crate::RustADBError;

/// Represents supported ADB reverse / forward protocols
#[derive(Clone, Debug)]
pub enum ADBProtoPort {
    /// TCP
    TCP(u16),
}

impl FromStr for ADBProtoPort {
    type Err = RustADBError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

impl TryFrom<&[u8]> for ADBProtoPort {
    type Error = RustADBError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let v = str::from_utf8(value)?;

        if let Some(port) = v.strip_prefix("tcp:") {
            // Remove trailing \0
            let port = port.trim_matches('\0');
            return Ok(Self::TCP(port.parse::<u16>()?));
        }

        Err(RustADBError::UnknownProtocol(v.to_string()))
    }
}

impl Display for ADBProtoPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ADBProtoPort::TCP(port) => write!(f, "tcp:{port}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ADBProtoPort;
    use std::str::FromStr;

    #[test]
    fn test_tcp_port_parsing() {
        let case = "tcp:1247";
        let proto = ADBProtoPort::from_str(&*case).expect("cannot parse input");

        match proto {
            ADBProtoPort::TCP(port) => {
                assert!(port == 1247);
            }
        }
    }

    #[test]
    fn test_wrong_tcp_port_parsing() {
        let case = "tcp:12A47";
        assert!(ADBProtoPort::from_str(&*case).is_err())
    }
}
