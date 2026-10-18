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

use adb_client::{ADBDeviceExt, ADBListItemType, HostFeatures};
use rand::{RngExt, SeedableRng, rngs::StdRng};

use crate::utils::{
    adb_file::{REMOTE_DIR, RemoteFile},
    get_connected_device,
};

/// Content spanning several 64KiB sync `DATA` chunks.
fn test_content() -> Vec<u8> {
    (0..300_000_u32)
        .map(|i| u8::try_from(i % 251).expect("value fits in u8"))
        .collect()
}

fn find_file<'a>(items: &'a [ADBListItemType], name: &str) -> Option<&'a ADBListItemType> {
    items
        .iter()
        .find(|item| matches!(item, ADBListItemType::File(file) if file.name == name))
}

/// Size of a sync `DATA` chunk, files around multiples of it are worth testing.
const CHUNK_SIZE: usize = 65535;

const MIB: usize = 1024 * 1024;

/// Seed of the random data, so that every run uses the same files.
const DEFAULT_SEED: u64 = 0xDEAD_BEEF_DEAD_BEEF;

/// Random data generator, seeded with [`DEFAULT_SEED`] unless `ADB_CLIENT_TEST_SEED` is set.
fn seeded_rng() -> StdRng {
    let seed = std::env::var("ADB_CLIENT_TEST_SEED").map_or(DEFAULT_SEED, |seed| {
        seed.parse().expect("ADB_CLIENT_TEST_SEED must be a u64")
    });
    println!("random data seed: {seed} (override with ADB_CLIENT_TEST_SEED)");

    StdRng::seed_from_u64(seed)
}

fn random_bytes(rng: &mut StdRng, len: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; len];
    rng.fill(&mut bytes[..]);
    bytes
}

/// Asserts both contents are equal, without printing them (they may be megabytes long).
fn assert_same_content(expected: &[u8], actual: &[u8], what: &str) {
    if let Some(index) = expected.iter().zip(actual).position(|(e, a)| e != a) {
        panic!("{what}: content differs at byte {index}");
    }
    assert_eq!(
        actual.len(),
        expected.len(),
        "{what}: content has a different length"
    );
}

#[test]
#[ignore = "requires a connected device"]
fn device_is_connected() {
    let mut device = get_connected_device();

    let features = device.host_features().expect("cannot get device features");
    assert!(
        features.contains(&HostFeatures::ShellV2),
        "device does not support shell v2: {features:?}"
    );
}

#[test]
#[ignore = "requires a connected device"]
fn shell_command_separates_stdout_and_stderr() {
    let mut device = get_connected_device();

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
fn shell_command_runs_as_shell_user() {
    let mut device = get_connected_device();

    let mut stdout = Vec::new();
    device
        .shell_command(&"id -un", Some(&mut stdout), None)
        .expect("cannot run shell command");

    let user = String::from_utf8_lossy(&stdout);
    assert!(
        ["shell", "root"].contains(&user.trim()),
        "unexpected user {user:?}"
    );
}

#[test]
#[ignore = "requires a connected device"]
fn file_commands_roundtrip() {
    let mut device = get_connected_device();
    let remote = RemoteFile::new("roundtrip.bin");
    let content = test_content();

    device
        .push(&mut content.as_slice(), &remote)
        .expect("cannot push file");

    let stat = device.stat(&remote).expect("cannot stat file");
    assert_eq!(stat.file_size as usize, content.len());

    let items = device.list(REMOTE_DIR).expect("cannot list directory");
    assert!(
        find_file(&items, remote.file_name()).is_some(),
        "pushed file not listed in {REMOTE_DIR}"
    );

    let mut pulled = Vec::new();
    device.pull(&remote, &mut pulled).expect("cannot pull file");
    assert!(pulled == content, "pulled content differs from pushed one");

    let stat_extended = device
        .stat_extended(&remote)
        .expect("cannot stat file")
        .expect("pushed file does not exist");
    assert_eq!(
        stat_extended.size,
        u64::try_from(content.len()).expect("length fits in u64")
    );
}

#[test]
#[ignore = "requires a connected device"]
fn missing_file_is_reported() {
    let mut device = get_connected_device();
    let missing = format!("{REMOTE_DIR}/adb_client_test_does_not_exist");

    // Sync STAT does not fail on missing files, but returns zeroed fields
    let stat = device.stat(&missing).expect("cannot stat file");
    assert_eq!(stat.file_perm, 0);

    assert!(
        device
            .stat_extended(&missing)
            .expect("cannot stat file")
            .is_none(),
        "missing file reported as existing"
    );

    let mut output = Vec::new();
    assert!(
        device.pull(&missing, &mut output).is_err(),
        "pulling a missing file must fail"
    );
}

#[test]
#[ignore = "requires a connected device"]
fn sync_session_runs_several_operations() {
    let mut device = get_connected_device();
    let first = RemoteFile::new("session_1.bin");
    let second = RemoteFile::new("session_2.bin");
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

#[cfg(feature = "framebuffer")]
#[test]
#[ignore = "requires a connected device"]
fn framebuffer_is_a_png_image() {
    const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

    let mut device = get_connected_device();
    let image = device.framebuffer_bytes().expect("cannot get framebuffer");

    assert!(image.starts_with(PNG_SIGNATURE), "framebuffer is not a PNG");
}

#[test]
#[ignore = "requires a connected device"]
fn push_pull_random_data() {
    let mut device = get_connected_device();
    let mut rng = seeded_rng();

    let sizes = [
        // Empty and tiny files
        0,
        1,
        // Around `DATA` chunk boundaries
        CHUNK_SIZE - 1,
        CHUNK_SIZE,
        CHUNK_SIZE + 1,
        2 * CHUNK_SIZE,
        2 * CHUNK_SIZE + 1,
        2 * MIB,
        // Large files, up to 100MiB
        10 * MIB,
        50 * MIB,
        100 * MIB,
    ];

    for size in sizes {
        let remote = RemoteFile::new(&format!("random_{size}.bin"));
        let content = random_bytes(&mut rng, size);

        device
            .push(&mut content.as_slice(), &remote)
            .unwrap_or_else(|e| panic!("cannot push {size} bytes: {e}"));

        let stat = device
            .stat(&remote)
            .expect("cannot stat {size} bytes file: {e}");
        assert_eq!(
            stat.file_size as usize, size,
            "stat size of {size} bytes file"
        );

        let mut pulled = Vec::new();
        device
            .pull(&remote, &mut pulled)
            .expect("cannot pull {size} bytes: {e}");
        assert_same_content(&content, &pulled, &format!("{size} bytes file"));
    }
}

#[test]
#[ignore = "requires a connected device"]
fn sync_session_push_pull_random_files() {
    let mut device = get_connected_device();
    let mut rng = seeded_rng();

    let files: Vec<(RemoteFile, Vec<u8>)> = (0..10)
        .map(|index| {
            let size = rng.random_range(0..512 * 1024);
            (
                RemoteFile::new(&format!("session_random_{index}.bin")),
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

#[test]
#[ignore = "requires a connected device"]
fn push_overwrites_with_smaller_content() {
    let mut device = get_connected_device();
    let mut rng = seeded_rng();
    let remote = RemoteFile::new("overwrite.bin");

    let large = random_bytes(&mut rng, 3 * CHUNK_SIZE);
    let small = random_bytes(&mut rng, 100);

    device
        .push(large.as_slice(), &remote)
        .expect("cannot push large content");
    device
        .push(small.as_slice(), &remote)
        .expect("cannot push small content");

    let mut pulled = Vec::new();
    device.pull(&remote, &mut pulled).expect("cannot pull file");
    assert_same_content(&small, &pulled, "overwritten file");
}
