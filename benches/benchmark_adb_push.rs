use adb_client::server::ADBServer;
#[cfg(feature = "usb")]
use adb_client::{ADBDeviceExt, usb::ADBUSBDevice};
use anyhow::Result;
use criterion::{BenchmarkId, Criterion, SamplingMode, criterion_group, criterion_main};
use rand::{RngExt, rng};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::Duration;

const LOCAL_TEST_FILE_PATH: &str = "test_file.bin";
const REMOTE_TEST_FILE_PATH: &str = "/data/local/tmp/test_file.bin";

/// Generate random test file with given size
fn generate_test_file(size_in_bytes: usize) -> Result<()> {
    const BUFFER_SIZE: usize = 64 * 1024;
    let mut buffer = vec![0u8; BUFFER_SIZE].into_boxed_slice();
    let mut test_file = File::create(LOCAL_TEST_FILE_PATH)?;

    let mut rng = rng();

    let mut remaining_bytes = size_in_bytes;

    while remaining_bytes > 0 {
        let bytes_to_write = remaining_bytes.min(BUFFER_SIZE);
        rng.fill(&mut buffer[..bytes_to_write]);
        test_file.write_all(&buffer[..bytes_to_write])?;
        remaining_bytes -= bytes_to_write;
    }

    Ok(())
}

/// Use `adb_client` crate to push a file on device
fn bench_adb_client_push() -> Result<()> {
    let mut client = ADBServer::default();
    let device = client.get_device()?;
    let f = File::open(LOCAL_TEST_FILE_PATH)?;
    Ok(device.push(f, REMOTE_TEST_FILE_PATH)?)
}

/// Use standard `adb` command to push a file on device
fn bench_adb_push_command() -> Result<()> {
    let output = Command::new("adb")
        .arg("push")
        .arg(LOCAL_TEST_FILE_PATH)
        .arg(REMOTE_TEST_FILE_PATH)
        .output()?;

    if !output.status.success() {
        eprintln!("error while starting adb push command");
    }
    Ok(())
}

/// Use `adb_client` crate to push a file on device directly over USB, without adb server
#[cfg(feature = "usb")]
fn bench_adb_client_usb_push(device: &ADBUSBDevice) -> Result<()> {
    let mut f = File::open(LOCAL_TEST_FILE_PATH)?;
    Ok(device.push(&mut f, &REMOTE_TEST_FILE_PATH)?)
}

/// Stop adb server, as it holds the USB interface of the device
#[cfg(feature = "usb")]
fn kill_adb_server() -> Result<()> {
    let output = Command::new("adb").arg("kill-server").output()?;

    if !output.status.success() {
        eprintln!("error while stopping adb server");
    }
    Ok(())
}

/// benchmarking `adb push INPUT DEST`, `adb_client` `ADBServerDevice.push(INPUT, DEST)`
/// and, with `usb` feature, `adb_client` `ADBUSBDevice.push(INPUT, DEST)`
fn benchmark_adb_push(c: &mut Criterion) {
    for (file_size, sample_size, measurement_time) in [
        (10 * 1024 * 1024, 100, 60),   // 10MB -> 100 iterations, 1 min
        (500 * 1024 * 1024, 10, 200),  // 500MB -> 10 iterations, ~3 min
        (1000 * 1024 * 1024, 10, 400), // 1GB -> 10 iterations, ~7 min
    ] {
        eprintln!("Benchmarking file_size={file_size} and sample_size={sample_size}");

        generate_test_file(file_size).expect("Cannot generate test file");

        let mut group = c.benchmark_group("ADB Push Benchmark");
        group.sample_size(sample_size);
        group.sampling_mode(SamplingMode::Flat);
        group.measurement_time(Duration::from_secs(measurement_time));

        group.bench_function(BenchmarkId::new("adb_client", "push"), |b| {
            b.iter(|| {
                bench_adb_client_push().expect("Error while benchmarking adb_client push");
            });
        });

        group.bench_function(BenchmarkId::new("adb", "push"), |b| {
            b.iter(|| {
                bench_adb_push_command().expect("Error while benchmarking adb push command");
            });
        });

        // adb server is automatically restarted by the server benchmarks of the next iteration
        #[cfg(feature = "usb")]
        {
            kill_adb_server().expect("Cannot stop adb server");
            let device = ADBUSBDevice::autodetect().expect("Cannot connect to USB device");

            group.bench_function(BenchmarkId::new("adb_client_usb", "push"), |b| {
                b.iter(|| {
                    bench_adb_client_usb_push(&device)
                        .expect("Error while benchmarking adb_client USB push");
                });
            });
        }

        group.finish();
    }
}

criterion_group!(benches, benchmark_adb_push);
criterion_main!(benches);
