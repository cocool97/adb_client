use std::fmt::Display;

#[derive(Debug)]
/// Type of reboot needed.
pub enum RebootType {
    /// "Classic" device reboot
    System,
    /// Reboots to bootloader
    Bootloader,
    /// Reboots to recovery
    Recovery,
    /// Reboots into recovery and automatically starts sideload mode
    Sideload,
    /// Same as `Sideload` but reboots after sideloading
    SideloadAutoReboot,
}

impl Display for RebootType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RebootType::System => write!(f, ""),
            RebootType::Bootloader => write!(f, "bootloader"),
            RebootType::Recovery => write!(f, "recovery"),
            RebootType::Sideload => write!(f, "sideload"),
            RebootType::SideloadAutoReboot => write!(f, "sideload-auto-reboot"),
        }
    }
}
