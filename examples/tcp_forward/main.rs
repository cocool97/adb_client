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

use std::io::{Read, Write};

use adb_client::{ADBLocalCommand, ADBUSBDevice};

fn main() -> adb_client::Result<()> {
    let mut device = ADBUSBDevice::autodetect()?;

    // Open a TCP session to port 8080 on the device
    let mut session = device
        .inner_mut()
        .open_session(&ADBLocalCommand::TcpConnect(8080))?;

    // Example: send an HTTP GET request
    let request = b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    session.get_transport_mut().write_all(request)?;

    // Read the response
    let mut response = Vec::new();
    session.get_transport_mut().read_to_end(&mut response)?;

    println!("Response ({} bytes):", response.len());
    println!("{}", String::from_utf8_lossy(&response));

    Ok(())
}
