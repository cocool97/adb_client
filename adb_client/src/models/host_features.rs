use std::fmt::Display;

/// Available host features.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum HostFeatures {
    /// Shell version 2.
    ShellV2,
    /// Command.
    Cmd,
}

impl HostFeatures {
    /// Parses a comma-separated list of features, as advertised by devices. Unknown features are skipped.
    pub(crate) fn parse_list(value: &[u8]) -> Vec<Self> {
        value
            .split(|x| x.eq(&b','))
            // Devices without any feature (older ones) advertise an empty list
            .filter(|v| !v.is_empty())
            .filter_map(|v| match Self::try_from(v) {
                Ok(feature) => Some(feature),
                Err(err) => {
                    // Devices advertise many features that are not used here
                    log::debug!("cannot create new feature: {err}");
                    None
                }
            })
            .collect()
    }

    /// Whether `features` allow shell service arguments (`shell,<args>:<command>`).
    ///
    /// Older devices (without `shell_v2` feature) only understand `shell:<command>`.
    pub(crate) fn supports_shell_v2(features: &[Self]) -> bool {
        features.contains(&Self::ShellV2) || features.contains(&Self::Cmd)
    }
}

impl Display for HostFeatures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShellV2 => write!(f, "ShellV2"),
            Self::Cmd => write!(f, "Cmd"),
        }
    }
}

impl TryFrom<&[u8]> for HostFeatures {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"shell_v2" => Ok(Self::ShellV2),
            b"cmd" => Ok(Self::Cmd),
            _ => Err(format!(
                "unknown feature {}",
                String::from_utf8_lossy(value)
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::HostFeatures;

    #[test]
    fn test_parse_features() {
        assert_eq!(
            HostFeatures::parse_list(b"abb,shell_v2,ls_v2,cmd,stat_v2"),
            [HostFeatures::ShellV2, HostFeatures::Cmd]
        );
    }

    #[test]
    fn test_parse_empty_features() {
        assert_eq!(HostFeatures::parse_list(b""), Vec::<HostFeatures>::new());
    }
}
