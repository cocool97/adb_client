use std::str::FromStr;

use crate::RustADBError;

pub enum AdbRequestStatus {
    Okay,
    Fail,
}

impl FromStr for AdbRequestStatus {
    type Err = RustADBError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lowercased = s.to_ascii_lowercase();
        match lowercased.as_str() {
            "okay" => Ok(Self::Okay),
            "fail" => Ok(Self::Fail),
            _ => Err(RustADBError::UnknownResultType(lowercased)),
        }
    }
}
