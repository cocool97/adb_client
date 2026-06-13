use std::fmt::Display;

use crate::RebootType;

/// ADB commands that relates to an actual device.
#[derive(Debug)]
pub enum ADBLocalCommand {
    /// Run a shell command with arguments
    ShellCommand(String, Vec<String>),
    /// Open an interactive shell session
    Shell,
    /// Execute a command without PTY allocation
    Exec(String),
    /// Start a sync session for file operations
    Sync,
    /// Reboot the device
    Reboot(RebootType),
    /// Set up port forwarding (remote, local)
    Forward(String, String),
    /// Remove a specific port forward
    ForwardRemove(String),
    /// Remove all port forwards
    ForwardRemoveAll,
    /// Set up reverse port forwarding (remote, local)
    Reverse(String, String),
    /// Remove a specific reverse forward
    ReverseRemove(String),
    /// Remove all reverse forwards
    ReverseRemoveAll,
    /// Reconnect to the device
    Reconnect,
    /// Remount the filesystem as read-write
    Remount,
    /// Disable dm-verity checking on userdebug builds
    DisableVerity,
    /// Re-enable dm-verity checking on userdebug builds
    EnableVerity,
    /// Uninstall a package, optionally for a specific user
    Uninstall(String, Option<String>),
    /// Install a package with the given size, optionally for a specific user
    Install(u64, Option<String>),
    /// Switch the device to TCP/IP mode on the given port
    TcpIp(u16),
    /// Switch the device back to USB mode
    Usb,
    /// Restart adbd with root permissions
    Root,
    /// Capture the device framebuffer
    #[cfg(feature = "framebuffer")]
    FrameBuffer,
    /// Open a TCP connection to a port on the device (formats to "tcp:<port>")
    TcpConnect(u16),
}

impl Display for ADBLocalCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sync => write!(f, "sync:"),
            Self::ShellCommand(command, shell_args) => {
                if shell_args.is_empty() {
                    // Shell v1: simple format for older ADB versions
                    write!(f, "shell:{command}")
                } else {
                    // Shell v2: with arguments
                    let args_s = shell_args.join(",");
                    write!(f, "shell,{args_s},raw:{command}")
                }
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
            Self::Install(size, user) => {
                write!(f, "exec:cmd package 'install'")?;
                if let Some(user) = user {
                    write!(f, " --user {user}")?;
                }
                write!(f, " -S {size}")
            }
            Self::Forward(remote, local) => {
                write!(f, "host:forward:{local};{remote}")
            }
            Self::ForwardRemove(local) => write!(f, "host:killforward:{local}"),
            Self::ForwardRemoveAll => write!(f, "host:killforward-all"),
            Self::Reverse(remote, local) => {
                write!(f, "reverse:forward:{remote};{local}")
            }
            Self::ReverseRemove(remote) => write!(f, "reverse:killforward:{remote}"),
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
            #[cfg(feature = "framebuffer")]
            Self::FrameBuffer => write!(f, "framebuffer:"),
            Self::TcpConnect(port) => write!(f, "tcp:{port}"),
        }
    }
}

#[test]
fn test_forward_remove_command() {
    let command = ADBLocalCommand::ForwardRemove("tcp:7100".to_string());

    assert_eq!(command.to_string(), "host:killforward:tcp:7100");
}

#[test]
fn test_reverse_remove_command() {
    let command = ADBLocalCommand::ReverseRemove("tcp:7100".to_string());

    assert_eq!(command.to_string(), "reverse:killforward:tcp:7100");
}
