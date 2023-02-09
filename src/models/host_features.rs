use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum HostFeatures {
    ShellV2,
    Cmd,
}

impl Display for HostFeatures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostFeatures::ShellV2 => write!(f, "ShellV2"),
            HostFeatures::Cmd => write!(f, "Cmd"),
        }
    }
}

impl TryFrom<&[u8]> for HostFeatures {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"shell_v2" => Ok(Self::ShellV2),
            b"cmd" => Ok(Self::Cmd),
            _ => Err(format!("Unknown value {value:?}")),
        }
    }
}
