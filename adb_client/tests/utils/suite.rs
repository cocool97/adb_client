//! Tests shared by every kind of device, run through [`ADBDeviceExt`] only.
//!
//! [`device_test_suite!`] declares them as `#[test]` functions in a test crate.

use std::{
    path::Path,
    time::{Duration, Instant},
};

use adb_client::ADBDeviceExt;

use crate::utils::{
    CHUNK_SIZE, MIB, adb_file::REMOTE_DIR, assert_same_content, find_file, random_bytes,
    seeded_rng, test_content, test_device::TestDevice,
};

/// Declares the shared tests, getting their device from `$get_device`.
macro_rules! device_test_suite {
    ($get_device:path) => {
        $crate::utils::suite::device_test_suite!(
            @tests $get_device,
            shell_command_writes_stdout,
            shell_command_runs_as_shell_user,
            commands_run_in_parallel,
            push_while_running_commands,
            file_commands_roundtrip,
            missing_file_is_reported,
            push_pull_random_data,
            push_overwrites_with_smaller_content,
            install_uninstall_apk,
        );

        #[cfg(feature = "framebuffer")]
        $crate::utils::suite::device_test_suite!(@tests $get_device, framebuffer_is_a_png_image);
    };
    (@tests $get_device:path, $($test:ident),+ $(,)?) => {
        $(
            #[test]
            #[ignore = "requires a connected device"]
            fn $test() {
                $crate::utils::suite::$test(&mut $get_device());
            }
        )+
    };
}
pub(crate) use device_test_suite;

pub(crate) fn shell_command_writes_stdout<D: ADBDeviceExt>(device: &TestDevice<D>) {
    let mut stdout = Vec::new();
    device
        .shell_command(&"echo out", Some(&mut stdout), None)
        .expect("cannot run shell command");

    assert_eq!(String::from_utf8_lossy(&stdout), "out\n");
}

pub(crate) fn shell_command_runs_as_shell_user<D: ADBDeviceExt>(device: &TestDevice<D>) {
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

/// A shell command completes while another one is still running on the same device.
pub(crate) fn commands_run_in_parallel<D: ADBDeviceExt + Sync>(device: &TestDevice<D>) {
    let start = Instant::now();

    std::thread::scope(|scope| {
        let sleep = scope.spawn(|| {
            device
                .shell_command(&"sleep 3", None, None)
                .expect("cannot run shell command");
            start.elapsed()
        });

        // Leave time to the first command to start
        std::thread::sleep(Duration::from_millis(500));
        let mut stdout = Vec::new();
        device
            .shell_command(&"echo parallel", Some(&mut stdout), None)
            .expect("cannot run shell command");
        let echo_done = start.elapsed();

        let sleep_done = sleep.join().expect("sleep thread panicked");
        assert_eq!(String::from_utf8_lossy(&stdout), "parallel\n");
        assert!(
            echo_done < sleep_done,
            "echo ended after {echo_done:?}, not before sleep which ended after {sleep_done:?}"
        );
    });
}

/// A large push and shell commands run at the same time, without mixing their data.
pub(crate) fn push_while_running_commands<D: ADBDeviceExt + Sync>(device: &mut TestDevice<D>) {
    let remote = device.remote_file("parallel_push.bin");
    let content = random_bytes(&mut seeded_rng(), 32 * MIB);
    let device: &D = device;

    std::thread::scope(|scope| {
        let push = scope.spawn(|| {
            device
                .push(&mut content.as_slice(), &remote)
                .expect("cannot push file");
        });

        while !push.is_finished() {
            let mut stdout = Vec::new();
            device
                .shell_command(&"echo parallel", Some(&mut stdout), None)
                .expect("cannot run shell command");
            assert_eq!(String::from_utf8_lossy(&stdout), "parallel\n");
        }
        push.join().expect("push thread panicked");
    });

    let mut pulled = Vec::new();
    device.pull(&remote, &mut pulled).expect("cannot pull file");
    assert_same_content(&content, &pulled, "file pushed while running commands");
}

pub(crate) fn file_commands_roundtrip<D: ADBDeviceExt>(device: &mut TestDevice<D>) {
    let remote = device.remote_file("roundtrip.bin");
    let content = test_content();

    device
        .push(&mut content.as_slice(), &remote)
        .expect("cannot push file");

    let stat = device.stat(&remote).expect("cannot stat file");
    assert_eq!(stat.file_size as usize, content.len());

    let items = device.list(&REMOTE_DIR).expect("cannot list directory");
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

pub(crate) fn missing_file_is_reported<D: ADBDeviceExt>(device: &TestDevice<D>) {
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

#[cfg(feature = "framebuffer")]
pub(crate) fn framebuffer_is_a_png_image<D: ADBDeviceExt>(device: &TestDevice<D>) {
    const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

    let image = device.framebuffer_bytes().expect("cannot get framebuffer");

    assert!(image.starts_with(PNG_SIGNATURE), "framebuffer is not a PNG");
}

pub(crate) fn push_pull_random_data<D: ADBDeviceExt>(device: &mut TestDevice<D>) {
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
        let remote = device.remote_file(&format!("random_{size}.bin"));
        let content = random_bytes(&mut rng, size);

        device
            .push(&mut content.as_slice(), &remote)
            .unwrap_or_else(|e| panic!("cannot push {size} bytes: {e}"));

        let stat = device
            .stat(&remote)
            .unwrap_or_else(|e| panic!("cannot stat {size} bytes file: {e}"));
        assert_eq!(
            stat.file_size as usize, size,
            "stat size of {size} bytes file"
        );

        let mut pulled = Vec::new();
        device
            .pull(&remote, &mut pulled)
            .unwrap_or_else(|e| panic!("cannot pull {size} bytes: {e}"));
        assert_same_content(&content, &pulled, &format!("{size} bytes file"));
    }
}

pub(crate) fn push_overwrites_with_smaller_content<D: ADBDeviceExt>(device: &mut TestDevice<D>) {
    let mut rng = seeded_rng();
    let remote = device.remote_file("overwrite.bin");

    let large = random_bytes(&mut rng, 3 * CHUNK_SIZE);
    let small = random_bytes(&mut rng, 100);

    device
        .push(&mut large.as_slice(), &remote)
        .expect("cannot push large content");
    device
        .push(&mut small.as_slice(), &remote)
        .expect("cannot push small content");

    let mut pulled = Vec::new();
    device.pull(&remote, &mut pulled).expect("cannot pull file");
    assert_same_content(&small, &pulled, "overwritten file");
}

/// Package of the APK used by [`install_uninstall_apk`].
const TEST_APK_PACKAGE: &str = "org.fossify.math";

/// Fossify Calculator APK, small and targeting a recent SDK, downloaded by `run_tests.sh` (not versioned).
fn test_apk_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/assets/fossify_calculator.apk")
}

fn is_package_installed<D: ADBDeviceExt>(device: &TestDevice<D>, package: &str) -> bool {
    let mut stdout = Vec::new();
    device
        .shell_command(
            &format!("pm list packages {package}"),
            Some(&mut stdout),
            None,
        )
        .expect("cannot list packages");

    // `pm list packages` filters on a substring, the exact package must be matched
    String::from_utf8_lossy(&stdout)
        .lines()
        .any(|line| line.trim() == format!("package:{package}"))
}

pub(crate) fn install_uninstall_apk<D: ADBDeviceExt>(device: &TestDevice<D>) {
    let apk_path = test_apk_path();
    assert!(
        apk_path.exists(),
        "{} is missing, run `run_tests.sh` to download it",
        apk_path.display()
    );

    // Uninstalling would delete the user's data of an app they actually use
    assert!(
        !is_package_installed(device, TEST_APK_PACKAGE),
        "{TEST_APK_PACKAGE} is already installed on the device, refusing to reinstall and uninstall it"
    );

    device.install(&apk_path, None).expect("cannot install APK");
    assert!(
        is_package_installed(device, TEST_APK_PACKAGE),
        "{TEST_APK_PACKAGE} not installed after install"
    );

    device
        .uninstall(&TEST_APK_PACKAGE, None)
        .expect("cannot uninstall package");
    assert!(
        !is_package_installed(device, TEST_APK_PACKAGE),
        "{TEST_APK_PACKAGE} still installed after uninstall"
    );
}
