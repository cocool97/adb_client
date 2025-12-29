use std::{fmt::Display, net::SocketAddrV4};

use crate::server::{WaitForDeviceState, WaitForDeviceTransport};

/// ADB commands that relates to the host and are handled by the ADB server.
pub(crate) enum ADBHostCommand {
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
    WaitForDevice(WaitForDeviceState, WaitForDeviceTransport),
}

impl Display for ADBHostCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ADBHostCommand::Version => write!(f, "host:version"),
            ADBHostCommand::Kill => write!(f, "host:kill"),
            ADBHostCommand::Devices => write!(f, "host:devices"),
            ADBHostCommand::DevicesLong => write!(f, "host:devices-l"),
            ADBHostCommand::TrackDevices => write!(f, "host:track-devices"),
            ADBHostCommand::TransportAny => write!(f, "host:transport-any"),
            ADBHostCommand::TransportSerial(serial) => write!(f, "host:transport:{serial}"),
            ADBHostCommand::Connect(addr) => write!(f, "host:connect:{addr}"),
            ADBHostCommand::Disconnect(addr) => write!(f, "host:disconnect:{addr}"),
            ADBHostCommand::Pair(addr, code) => {
                write!(f, "host:pair:{code}:{addr}")
            }
            ADBHostCommand::MDNSCheck => write!(f, "host:mdns:check"),
            ADBHostCommand::MDNSServices => write!(f, "host:mdns:services"),
            ADBHostCommand::ServerStatus => write!(f, "host:server-status"),
            ADBHostCommand::ReconnectOffline => write!(f, "host:reconnect-offline"),
            ADBHostCommand::WaitForDevice(wait_for_device_state, wait_for_device_transport) => {
                write!(
                    f,
                    "host:wait-for-{wait_for_device_transport}-{wait_for_device_state}"
                )
            }
            ADBHostCommand::HostFeatures => write!(f, "host:features"),
        }
    }
}
