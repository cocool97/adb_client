use adb_client::ADBServer;
use anyhow::Result;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::{Rng, rng};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::Duration;

const LOCAL_TEST_FILE_PATH: &str = "test_file.bin";
const REMOTE_TEST_FILE_PATH: &str = "/data/local/tmp/test_file.bin";

/// Generate random test file with given size
fn generate_test_file(size_in_bytes: usize) -> Result<()> {
    let mut test_file = File::create(LOCAL_TEST_FILE_PATH)?;

    let mut rng = rng();

    const BUFFER_SIZE: usize = 64 * 1024;
    let mut buffer = [0u8; BUFFER_SIZE];
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
    let mut device = client.get_device()?;
    let f = File::open(LOCAL_TEST_FILE_PATH)?;
    Ok(device.push(f, REMOTE_TEST_FILE_PATH)?)
}

/// Use standard `adb` command ti push a file on device
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

/// benchmarking `adb push INPUT DEST` and adb_client `ADBServerDevice.push(INPUT, DEST)`
fn benchmark_adb_push(c: &mut Criterion) {
    for (file_size, sample_size) in [
        (10 * 1024 * 1024, 100),  // 10MB -> 100 iterations
        (500 * 1024 * 1024, 50),  // 500MB -> 50 iterations
        (1000 * 1024 * 1024, 20), // 1GB -> 20 iterations
    ] {
        eprintln!(
            "Benchmarking file_size={} and sample_size={}",
            file_size, sample_size
        );

        generate_test_file(file_size).expect("Cannot generate test file");

        let mut group = c.benchmark_group("ADB Push Benchmark");
        group.sample_size(sample_size);

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

        group.finish();
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(1000));
    targets = benchmark_adb_push
);
criterion_main!(benches);
