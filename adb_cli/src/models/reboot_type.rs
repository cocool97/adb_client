use adb_client::RebootType;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum RebootTypeCommand {
    System,
    Bootloader,
    Recovery,
    Sideload,
    SideloadAutoReboot,
    Fastboot,
}

impl From<RebootTypeCommand> for RebootType {
    fn from(value: RebootTypeCommand) -> Self {
        match value {
            RebootTypeCommand::System => Self::System,
            RebootTypeCommand::Bootloader => Self::Bootloader,
            RebootTypeCommand::Recovery => Self::Recovery,
            RebootTypeCommand::Sideload => Self::Sideload,
            RebootTypeCommand::SideloadAutoReboot => Self::SideloadAutoReboot,
            RebootTypeCommand::Fastboot => Self::Fastboot,
        }
    }
}
