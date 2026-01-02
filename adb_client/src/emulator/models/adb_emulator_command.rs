use std::fmt::Display;

pub enum ADBEmulatorCommand {
    Authenticate(String),
    Sms(String, String),
    Rotate,
}

impl Display for ADBEmulatorCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Need to call `writeln!` because emulator commands are '\n' terminated
        match self {
            Self::Authenticate(token) => writeln!(f, "auth {token}"),
            Self::Sms(phone_number, content) => {
                writeln!(f, "sms send {phone_number} {content}")
            }
            Self::Rotate => writeln!(f, "rotate"),
        }
    }
}

impl ADBEmulatorCommand {
    /// Return the number of lines to skip per command when checking its result
    pub(crate) const fn skip_response_lines(&self) -> u8 {
        match self {
            Self::Authenticate(_) => 1,
            Self::Sms(_, _) | Self::Rotate => 0,
        }
    }
}
