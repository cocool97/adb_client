use std::fmt::Display;

use crate::{WaitForDeviceState, WaitForDeviceTransport};

use super::RebootType;
use std::net::SocketAddrV4;

pub(crate) enum AdbServerCommand {
    // Host commands
    Version,
    Kill,
    Devices,
    DevicesLong,
    TrackDevices,
    HostFeatures,
    Connect(SocketAddrV4),
    Disconnect(SocketAddrV4),
    Pair(SocketAddrV4, String),
    TransportAny,
    TransportSerial(String),
    MDNSCheck,
    MDNSServices,
    ServerStatus,
    ReconnectOffline,
    Uninstall(String),
    Install(u64),
    WaitForDevice(WaitForDeviceState, WaitForDeviceTransport),
    // Local commands
    ShellCommand(String),
    Shell,
    FrameBuffer,
    Sync,
    Reboot(RebootType),
    Forward(String, String),
    ForwardRemoveAll,
    Reverse(String, String),
    ReverseRemoveAll,
    Reconnect,
    TcpIp(u16),
    Usb,
}

impl Display for AdbServerCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdbServerCommand::Version => write!(f, "host:version"),
            AdbServerCommand::Kill => write!(f, "host:kill"),
            AdbServerCommand::Devices => write!(f, "host:devices"),
            AdbServerCommand::DevicesLong => write!(f, "host:devices-l"),
            AdbServerCommand::Sync => write!(f, "sync:"),
            AdbServerCommand::TrackDevices => write!(f, "host:track-devices"),
            AdbServerCommand::TransportAny => write!(f, "host:transport-any"),
            AdbServerCommand::TransportSerial(serial) => write!(f, "host:transport:{serial}"),
            AdbServerCommand::ShellCommand(command) => match std::env::var("TERM") {
                Ok(term) => write!(f, "shell,TERM={term},raw:{command}"),
                Err(_) => write!(f, "shell,raw:{command}"),
            },
            AdbServerCommand::Shell => match std::env::var("TERM") {
                Ok(term) => write!(f, "shell,TERM={term},raw:"),
                Err(_) => write!(f, "shell,raw:"),
            },
            AdbServerCommand::HostFeatures => write!(f, "host:features"),
            AdbServerCommand::Reboot(reboot_type) => {
                write!(f, "reboot:{reboot_type}")
            }
            AdbServerCommand::Connect(addr) => write!(f, "host:connect:{}", addr),
            AdbServerCommand::Disconnect(addr) => write!(f, "host:disconnect:{}", addr),
            AdbServerCommand::Pair(addr, code) => {
                write!(f, "host:pair:{code}:{addr}")
            }
            AdbServerCommand::FrameBuffer => write!(f, "framebuffer:"),
            AdbServerCommand::Forward(remote, local) => {
                write!(f, "host:forward:{local};{remote}")
            }
            AdbServerCommand::ForwardRemoveAll => write!(f, "host:killforward-all"),
            AdbServerCommand::Reverse(remote, local) => {
                write!(f, "reverse:forward:{remote};{local}")
            }
            AdbServerCommand::ReverseRemoveAll => write!(f, "reverse:killforward-all"),
            AdbServerCommand::MDNSCheck => write!(f, "host:mdns:check"),
            AdbServerCommand::MDNSServices => write!(f, "host:mdns:services"),
            AdbServerCommand::ServerStatus => write!(f, "host:server-status"),
            AdbServerCommand::Reconnect => write!(f, "reconnect"),
            AdbServerCommand::ReconnectOffline => write!(f, "host:reconnect-offline"),
            AdbServerCommand::TcpIp(port) => {
                write!(f, "tcpip:{port}")
            }
            AdbServerCommand::Usb => write!(f, "usb:"),
            AdbServerCommand::Install(size) => write!(f, "exec:cmd package 'install' -S {size}"),
            AdbServerCommand::Uninstall(package) => {
                write!(f, "exec:cmd package 'uninstall' {package}")
            }
            AdbServerCommand::WaitForDevice(wait_for_device_state, wait_for_device_transport) => {
                write!(
                    f,
                    "host:wait-for-{wait_for_device_transport}-{wait_for_device_state}"
                )
            }
        }
    }
}

#[test]
fn test_pair_command() {
    let host = "192.168.0.197:34783";
    let code = "091102";
    let code_u32 = code.parse::<u32>().expect("cannot parse u32");
    let pair = AdbServerCommand::Pair(host.parse().expect("cannot parse host"), code.into());

    assert_eq!(pair.to_string(), format!("host:pair:{code}:{host}"));
    assert_ne!(pair.to_string(), format!("host:pair:{code_u32}:{host}"))
}
