use clap::Parser;

#[derive(Parser, Debug)]
pub enum EmuCommand {
    /// Send a SMS with given phone number and given content
    Sms {
        phone_number: String,
        content: String,
    },
    /// Rotate device screen from 90°
    Rotate,
}
