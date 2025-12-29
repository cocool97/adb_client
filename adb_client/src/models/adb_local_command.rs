use std::fmt::Display;

use crate::RebootType;

/// ADB commands that relates to an actual device.
pub(crate) enum ADBLocalCommand {
    ShellCommand(String, Vec<String>),
    Shell,
    Exec(String),
    FrameBuffer,
    Sync,
    Reboot(RebootType),
    Forward(String, String),
    ForwardRemoveAll,
    Reverse(String, String),
    ReverseRemoveAll,
    Reconnect,
    Remount,
    DisableVerity,
    EnableVerity,
    Uninstall(String),
    Install(u64),
    TcpIp(u16),
    Usb,
}

impl Display for ADBLocalCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ADBLocalCommand::Sync => write!(f, "sync:"),
            ADBLocalCommand::ShellCommand(command, shell_args) => {
                let args_s = shell_args.join(",");
                write!(
                    f,
                    "shell{}{args_s},raw:{command}",
                    if shell_args.is_empty() { "" } else { "," }
                )
            }
            ADBLocalCommand::Shell => match std::env::var("TERM") {
                Ok(term) => write!(f, "shell,TERM={term},raw:"),
                Err(_) => write!(f, "shell,raw:"),
            },
            ADBLocalCommand::Exec(command) => write!(f, "exec:{command}"),

            ADBLocalCommand::Reboot(reboot_type) => {
                write!(f, "reboot:{reboot_type}")
            }
            ADBLocalCommand::Uninstall(package) => {
                write!(f, "exec:cmd package 'uninstall' {package}")
            }
            ADBLocalCommand::FrameBuffer => write!(f, "framebuffer:"),
            ADBLocalCommand::Install(size) => write!(f, "exec:cmd package 'install' -S {size}"),
            ADBLocalCommand::Forward(remote, local) => {
                write!(f, "host:forward:{local};{remote}")
            }
            ADBLocalCommand::ForwardRemoveAll => write!(f, "host:killforward-all"),
            ADBLocalCommand::Reverse(remote, local) => {
                write!(f, "reverse:forward:{remote};{local}")
            }
            ADBLocalCommand::ReverseRemoveAll => write!(f, "reverse:killforward-all"),

            ADBLocalCommand::Reconnect => write!(f, "reconnect"),
            ADBLocalCommand::Remount => write!(f, "remount:"),
            ADBLocalCommand::DisableVerity => write!(f, "disable-verity:"),
            ADBLocalCommand::EnableVerity => write!(f, "enable-verity:"),
            ADBLocalCommand::TcpIp(port) => {
                write!(f, "tcpip:{port}")
            }
            ADBLocalCommand::Usb => write!(f, "usb:"),
        }
    }
}
