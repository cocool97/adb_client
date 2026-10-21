//! Integration tests running against a real device, directly over USB.
//!
//! They need a device connected over USB with the host key authorized, and no ADB server running (it would
//! claim the device first: stop it with `adb kill-server`). They are therefore ignored by default. Run them with:
//!
//! ```sh
//! cargo test -p adb_client --features usb --test full_usb_device -- --ignored
//! ```
//!
//! The device is autodetected, which fails if several are connected. Set `ADB_CLIENT_TEST_USB_ID` to
//! `vendor_id:product_id` (in hexadecimal, as `lsusb` prints it) to choose one.
//!
//! The USB interface can only be claimed once, so tests run one after the other.
//!
//! Tests only work in their own files under `/data/local/tmp`, removed once done.
//!
//! Tests using random data are seeded with a fixed value, so every run uses the same data. Set
//! `ADB_CLIENT_TEST_SEED` (a `u64`) to use another one.

#![cfg(feature = "usb")]

mod utils;

use crate::utils::{get_connected_usb_device, suite::device_test_suite};

device_test_suite!(get_connected_usb_device);
