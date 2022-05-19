#[derive(Debug)]
pub enum HostFeatures {
    ShellV2,
    Cmd,
}

impl TryFrom<&[u8]> for HostFeatures {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"shell_v2" => Ok(Self::ShellV2),
            b"cmd" => Ok(Self::Cmd),
            _ => Err(format!("Unknown value {:?}", value)),
        }
    }
}
