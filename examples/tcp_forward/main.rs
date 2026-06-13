//! Example: open a raw TCP session to a device port via USB transport.
//!
//! This demonstrates how to use `open_session` with `ADBLocalCommand::TcpConnect`
//! to establish a bidirectional stream to a TCP port on the device — the building
//! block for implementing `adb forward` semantics without an intermediate adb server.
//!
//! Usage:
//!   cargo run -p tcp_forward
//!
//! Make sure a service is listening on port 8080 on the device before running.
//! For example, run `nc -l -p 8080` on the device via `adb shell`.

use adb_client::{ADBLocalCommand, usb::ADBUSBDevice};

fn main() -> adb_client::Result<()> {
    let mut device = ADBUSBDevice::autodetect()?;

    // Open a TCP session to port 8080 on the device.
    // This is equivalent to the service string "tcp:8080" in the ADB protocol.
    let session = device
        .inner_mut()
        .open_session(&ADBLocalCommand::TcpConnect(8080))?;

    println!("Session opened successfully!");
    println!("  local_id:  {}", session.local_id());
    println!("  remote_id: {}", session.remote_id());

    // At this point you have a live ADB session connected to tcp:8080 on the device.
    // Use `session.get_transport_mut()` to read/write ADB protocol messages
    // via `read_message()` and `write_message()`.

    // The session is automatically closed (CLSE) when dropped.
    drop(session);

    println!("Session closed.");
    Ok(())
}
