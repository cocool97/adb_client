//! Run several commands at the same time on a device connected over USB.
//!
//! A long `sleep 30` runs in a shell while a file is pushed, and while `id` runs in another shell: each command
//! gets its own session on the same USB connection, so none of them waits for the others.
//!
//! No ADB server must be running, it would claim the device first (stop it with `adb kill-server`).

use std::{io::Cursor, thread, time::Instant};

use adb_client::{ADBDeviceExt, Result, usb::ADBUSBDevice};

const REMOTE_PATH: &str = "/data/local/tmp/adb_client_sessions_example";
const PUSH_SIZE: usize = 64 * 1024 * 1024;

fn main() -> Result<()> {
    let device = ADBUSBDevice::autodetect()?;
    let start = Instant::now();
    let log = |message: &str| println!("[{:>6.2}s] {message}", start.elapsed().as_secs_f32());

    thread::scope(|scope| {
        let sleep = scope.spawn(|| -> Result<()> {
            log("sleep 30: started");
            device.shell_command(&"sleep 30", None, None)?;
            log("sleep 30: done");
            Ok(())
        });

        let push = scope.spawn(|| -> Result<()> {
            log(&format!("push of {} MiB: started", PUSH_SIZE / 1024 / 1024));
            let mut data = Cursor::new(vec![0x42; PUSH_SIZE]);
            device.push(&mut data, &REMOTE_PATH)?;
            log("push: done");
            Ok(())
        });

        log("id: started");
        let mut output = Vec::new();
        device.shell_command(&"id", Some(&mut output), None)?;
        log(&format!("id: {}", String::from_utf8_lossy(&output).trim()));

        push.join().expect("push thread panicked")?;
        sleep.join().expect("sleep thread panicked")
    })?;

    device.shell_command(&format!("rm {REMOTE_PATH}"), None, None)?;
    Ok(())
}
