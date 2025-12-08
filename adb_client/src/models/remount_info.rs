use crate::Result;

#[derive(Debug)]
/// Information about remount operation
pub struct RemountInfo {
    /// Path that was remounted
    pub path: String,
    /// Mode that was used for remounting
    pub mode: String,
}

impl RemountInfo {
    pub(crate) fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() == 4 && parts[0] == "Using" && parts[2] == "for" {
            Ok(RemountInfo {
                path: parts[1].to_string(),
                mode: parts[3].to_string(),
            })
        } else {
            Err(crate::RustADBError::RemountError(s.to_string()))
        }
    }

    pub(crate) fn from_str_response(s: &str) -> Result<Vec<Self>> {
        if !s.ends_with("remount succeeded") {
            return Err(crate::RustADBError::RemountError(s.to_string()));
        }

        let mut infos = Vec::new();
        for line in s.lines() {
            if line.starts_with("Using") {
                infos.push(Self::from_str(line)?);
            }
        }

        Ok(infos)
    }
}
