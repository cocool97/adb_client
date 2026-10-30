//! Integration tests running against a real device, through a local ADB server.
//!
//! They need a device connected and authorized (`adb devices` shows it in `device` state), and are therefore
//! ignored by default. Run them with:
//!
//! ```sh
//! cargo test -p adb_client --test full_server_device -- --ignored
//! ```
//!
//! If several devices are connected, the first one in `device` state is used. Set `ADB_CLIENT_TEST_SERIAL`
//! to choose another one.
//!
//! Tests only work in their own files under `/data/local/tmp`, removed once done.
//!
//! Tests using random data are seeded with a fixed value, so every run uses the same data. Set
//! `ADB_CLIENT_TEST_SEED` (a `u64`) to use another one.

mod utils;

use adb_client::{ADBDeviceExt, HostFeatures};
use rand::RngExt;

use crate::utils::{
    adb_file::REMOTE_DIR, assert_same_content, find_file, get_connected_server_device,
    random_bytes, seeded_rng, suite::device_test_suite, test_content,
};

device_test_suite!(get_connected_server_device);

#[test]
#[ignore = "requires a connected device"]
fn device_is_connected() {
    let device = get_connected_server_device();

    let features = device.host_features().expect("cannot get device features");
    assert!(
        features.contains(&HostFeatures::ShellV2),
        "device does not support shell v2: {features:?}"
    );
}

#[test]
#[ignore = "requires a connected device"]
fn shell_command_separates_stdout_and_stderr() {
    let device = get_connected_server_device();

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    device
        .shell_command(
            &"echo out; echo err >&2",
            Some(&mut stdout),
            Some(&mut stderr),
        )
        .expect("cannot run shell command");

    assert_eq!(String::from_utf8_lossy(&stdout), "out\n");
    assert_eq!(String::from_utf8_lossy(&stderr), "err\n");
}

#[test]
#[ignore = "requires a connected device"]
fn sync_session_runs_several_operations() {
    let mut device = get_connected_server_device();
    let first = device.remote_file("session_1.bin");
    let second = device.remote_file("session_2.bin");
    let content = test_content();

    let mut sync = device.sync_session().expect("cannot open sync session");

    sync.push(&mut content.as_slice(), first.as_ref())
        .expect("cannot push first file");
    sync.push(&mut &b"second"[..], second.as_ref())
        .expect("cannot push second file");

    let items = sync.list(REMOTE_DIR).expect("cannot list directory");
    for file in [&first, &second] {
        assert!(
            find_file(&items, file.file_name()).is_some(),
            "{} not listed",
            file.as_ref()
        );
    }

    let mut pulled = Vec::new();
    sync.pull(first.as_ref(), &mut pulled)
        .expect("cannot pull first file");
    assert!(pulled == content, "pulled content differs from pushed one");

    assert_eq!(
        sync.stat(second.as_ref())
            .expect("cannot stat second file")
            .file_size,
        6
    );

    // A failure ends the session
    assert!(
        sync.pull(
            &format!("{REMOTE_DIR}/adb_client_test_does_not_exist"),
            &mut Vec::new()
        )
        .is_err(),
        "pulling a missing file must fail"
    );
    assert!(
        sync.stat(first.as_ref()).is_err(),
        "session must be closed after a failure"
    );
}

#[test]
#[ignore = "requires a connected device"]
fn sync_session_push_pull_random_files() {
    let mut device = get_connected_server_device();
    let mut rng = seeded_rng();

    let files: Vec<_> = (0..10)
        .map(|index| {
            let size = rng.random_range(0..512 * 1024);
            (
                device.remote_file(&format!("session_random_{index}.bin")),
                random_bytes(&mut rng, size),
            )
        })
        .collect();

    let mut sync = device.sync_session().expect("cannot open sync session");

    // All pushes, then all pulls: the connection must stay in sync across many transfers
    for (remote, content) in &files {
        sync.push(&mut content.as_slice(), remote.as_ref())
            .unwrap_or_else(|e| panic!("cannot push {}: {e}", remote.as_ref()));
    }
    for (remote, content) in &files {
        let mut pulled = Vec::new();
        sync.pull(remote.as_ref(), &mut pulled)
            .unwrap_or_else(|e| panic!("cannot pull {}: {e}", remote.as_ref()));
        assert_same_content(content, &pulled, remote.as_ref());
    }

    sync.quit().expect("cannot end sync session");
}
