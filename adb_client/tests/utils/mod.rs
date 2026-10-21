// Each test suite only uses part of these helpers
#![expect(dead_code)]

pub(crate) mod adb_file;
pub(crate) mod suite;
pub(crate) mod test_device;

use adb_client::{
    ADBListItemType,
    server::{ADBServer, DeviceState},
    server_device::ADBServerDevice,
};
use rand::{RngExt, SeedableRng, rngs::StdRng};

use crate::utils::test_device::TestDevice;

/// Returns the device under test through the local ADB server, asserting it is connected.
pub(crate) fn get_connected_server_device() -> TestDevice<ADBServerDevice> {
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

    let device = server
        .get_device_by_name(&serial)
        .unwrap_or_else(|e| panic!("cannot get device {serial}: {e}"));
    TestDevice::new(device)
}

/// Returns the device under test over USB.
///
/// The USB interface can only be claimed once: tests using it run one after the other, and no ADB server
/// must be running.
#[cfg(feature = "usb")]
pub(crate) fn get_connected_usb_device() -> TestDevice<adb_client::usb::ADBUSBDevice> {
    use adb_client::usb::ADBUSBDevice;
    use std::sync::{Mutex, PoisonError};

    static USB_LOCK: Mutex<()> = Mutex::new(());
    // A failed test poisons the lock, which does not matter for the next ones
    let lock = USB_LOCK.lock().unwrap_or_else(PoisonError::into_inner);

    let device = match std::env::var("ADB_CLIENT_TEST_USB_ID") {
        Ok(id) => {
            let (vendor_id, product_id) = id
                .split_once(':')
                .and_then(|(vendor_id, product_id)| {
                    Some((
                        u16::from_str_radix(vendor_id, 16).ok()?,
                        u16::from_str_radix(product_id, 16).ok()?,
                    ))
                })
                .expect("ADB_CLIENT_TEST_USB_ID must be `vendor_id:product_id` in hexadecimal");
            ADBUSBDevice::new(vendor_id, product_id)
        }
        Err(_) => ADBUSBDevice::autodetect(),
    }
    .unwrap_or_else(|e| {
        panic!("cannot open USB device (is an ADB server still running? `adb kill-server`): {e}")
    });

    TestDevice::exclusive(device, lock)
}

/// Content spanning several 64KiB sync `DATA` chunks.
pub(crate) fn test_content() -> Vec<u8> {
    (0..300_000_u32)
        .map(|i| u8::try_from(i % 251).expect("value fits in u8"))
        .collect()
}

pub(crate) fn find_file<'a>(
    items: &'a [ADBListItemType],
    name: &str,
) -> Option<&'a ADBListItemType> {
    items
        .iter()
        .find(|item| matches!(item, ADBListItemType::File(file) if file.name == name))
}

/// Size of a sync `DATA` chunk, files around multiples of it are worth testing.
pub(crate) const CHUNK_SIZE: usize = 65535;

pub(crate) const MIB: usize = 1024 * 1024;

/// Seed of the random data, so that every run uses the same files.
const DEFAULT_SEED: u64 = 0xDEAD_BEEF_DEAD_BEEF;

/// Random data generator, seeded with [`DEFAULT_SEED`] unless `ADB_CLIENT_TEST_SEED` is set.
pub(crate) fn seeded_rng() -> StdRng {
    let seed = std::env::var("ADB_CLIENT_TEST_SEED").map_or(DEFAULT_SEED, |seed| {
        seed.parse().expect("ADB_CLIENT_TEST_SEED must be a u64")
    });
    println!("random data seed: {seed} (override with ADB_CLIENT_TEST_SEED)");

    StdRng::seed_from_u64(seed)
}

pub(crate) fn random_bytes(rng: &mut StdRng, len: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; len];
    rng.fill(&mut bytes[..]);
    bytes
}

/// Asserts both contents are equal, without printing them (they may be megabytes long).
pub(crate) fn assert_same_content(expected: &[u8], actual: &[u8], what: &str) {
    if let Some(index) = expected.iter().zip(actual).position(|(e, a)| e != a) {
        panic!("{what}: content differs at byte {index}");
    }
    assert_eq!(
        actual.len(),
        expected.len(),
        "{what}: content has a different length"
    );
}
