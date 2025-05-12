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
            RebootTypeCommand::System => RebootType::System,
            RebootTypeCommand::Bootloader => RebootType::Bootloader,
            RebootTypeCommand::Recovery => RebootType::Recovery,
            RebootTypeCommand::Sideload => RebootType::Sideload,
            RebootTypeCommand::SideloadAutoReboot => RebootType::SideloadAutoReboot,
            RebootTypeCommand::Fastboot => RebootType::Fastboot,
        }
    }
}
