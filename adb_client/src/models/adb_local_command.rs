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
    Uninstall(String, Option<String>),
    Install(u64),
    TcpIp(u16),
    Usb,
    Root,
}

impl Display for ADBLocalCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sync => write!(f, "sync:"),
            Self::ShellCommand(command, shell_args) => {
                let args_s = shell_args.join(",");
                write!(
                    f,
                    "shell{}{args_s},raw:{command}",
                    if shell_args.is_empty() { "" } else { "," }
                )
            }
            Self::Shell => match std::env::var("TERM") {
                Ok(term) => write!(f, "shell,TERM={term},raw:"),
                Err(_) => write!(f, "shell,raw:"),
            },
            Self::Exec(command) => write!(f, "exec:{command}"),
            Self::Reboot(reboot_type) => {
                write!(f, "reboot:{reboot_type}")
            }
            Self::Uninstall(package, user) => {
                write!(f, "exec:cmd package 'uninstall'")?;
                if let Some(user) = user {
                    write!(f, " --user {user}")?;
                }
                write!(f, " {package}")
            }
            Self::FrameBuffer => write!(f, "framebuffer:"),
            Self::Install(size) => write!(f, "exec:cmd package 'install' -S {size}"),
            Self::Forward(remote, local) => {
                write!(f, "host:forward:{local};{remote}")
            }
            Self::ForwardRemoveAll => write!(f, "host:killforward-all"),
            Self::Reverse(remote, local) => {
                write!(f, "reverse:forward:{remote};{local}")
            }
            Self::ReverseRemoveAll => write!(f, "reverse:killforward-all"),
            Self::Reconnect => write!(f, "reconnect"),
            Self::Remount => write!(f, "remount:"),
            Self::DisableVerity => write!(f, "disable-verity:"),
            Self::EnableVerity => write!(f, "enable-verity:"),
            Self::TcpIp(port) => {
                write!(f, "tcpip:{port}")
            }
            Self::Usb => write!(f, "usb:"),
            Self::Root => write!(f, "root:"),
        }
    }
}
