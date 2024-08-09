use clap::Parser;

#[derive(Parser, Debug)]
pub enum EmuCommand {
    /// Sends a SMS with given phone number and given content
    Sms {
        phone_number: String,
        content: String,
    },
    /// Rotates device screen from 90°
    Rotate,
}
