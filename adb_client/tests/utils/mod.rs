pub(crate) mod adb_file;

use adb_client::{
    server::{ADBServer, DeviceState},
    server_device::ADBServerDevice,
};

/// Returns the device under test, asserting it is connected.
pub(crate) fn get_connected_device() -> ADBServerDevice {
    let mut server = ADBServer::default();

    let version = server.version().expect("cannot get ADB server version");
    assert_eq!(version.major, 1, "unexpected ADB server version {version}");

    let devices = server.devices().expect("cannot list devices");
    let serial = match std::env::var("ADB_CLIENT_TEST_SERIAL") {
        Ok(serial) => serial,
        Err(_) => devices
            .iter()
            .find(|device| device.state == DeviceState::Device)
            .unwrap_or_else(|| panic!("no device connected in `device` state: {devices:?}"))
            .identifier
            .clone(),
    };
    assert!(
        devices
            .iter()
            .any(|device| device.identifier == serial && device.state == DeviceState::Device),
        "device {serial} is not connected in `device` state: {devices:?}"
    );

    server
        .get_device_by_name(&serial)
        .unwrap_or_else(|e| panic!("cannot get device {serial}: {e}"))
}
