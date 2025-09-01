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
            ADBEmulatorCommand::Authenticate(token) => writeln!(f, "auth {token}"),
            ADBEmulatorCommand::Sms(phone_number, content) => {
                writeln!(f, "sms send {phone_number} {content}")
            }
            ADBEmulatorCommand::Rotate => writeln!(f, "rotate"),
        }
    }
}

impl ADBEmulatorCommand {
    /// Return the number of lines to skip per command when checking its result
    pub(crate) fn skip_response_lines(&self) -> u8 {
        match self {
            ADBEmulatorCommand::Authenticate(_) => 1,
            ADBEmulatorCommand::Sms(_, _) => 0,
            ADBEmulatorCommand::Rotate => 0,
        }
    }
}
