//! Forward and reverse forward connections over USB, without any ADB server.
//!
//! 1. Reverse: a host server is exposed on a device port, checked to be listening on the device.
//! 2. Forward: this device port is exposed on a host port, and reached from the host.
//!
//! Connections to the host port go through the device and come back to the host server: data is checked both ways,
//! without any tool needed on the device.
//!
//! No ADB server must be running, it would claim the device first (stop it with `adb kill-server`).

use std::{
    io::{BufRead, BufReader, Write},
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::mpsc,
    thread,
    time::Duration,
};

use adb_client::{ADBDeviceExt, Result, RustADBError, usb::ADBUSBDevice};

/// Device port exposing the host server.
const DEVICE_PORT: u16 = 27_301;
/// Host port exposing the device port.
const HOST_PORT: u16 = 27_302;
/// Maximum time to wait for an answer.
const TIMEOUT: Duration = Duration::from_secs(10);

fn main() -> Result<()> {
    let device = ADBUSBDevice::autodetect()?;

    // Host server: answer `pong` to `ping`
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    let server_port = listener.local_addr()?.port();
    let (requests_sender, requests) = mpsc::channel();
    thread::spawn(move || {
        let _ = requests_sender.send(serve(&listener));
    });

    // 1. Expose the host server on the device
    device.reverse(&format!("tcp:{DEVICE_PORT}"), &format!("tcp:{server_port}"))?;
    println!("reverse: device tcp:{DEVICE_PORT} -> host tcp:{server_port}");

    let mut listening = Vec::new();
    device.shell_command(&"netstat -ltn", Some(&mut listening), None)?;
    let listening = String::from_utf8_lossy(&listening);
    if !listening.contains(&format!(":{DEVICE_PORT} ")) {
        return Err(RustADBError::ADBRequestFailed(format!(
            "device tcp:{DEVICE_PORT} is not listening:\n{listening}"
        )));
    }
    println!("device tcp:{DEVICE_PORT} is listening");

    // 2. Expose the device port on the host, and reach it
    device.forward(&format!("tcp:{DEVICE_PORT}"), &format!("tcp:{HOST_PORT}"))?;
    println!("forward: host tcp:{HOST_PORT} -> device tcp:{DEVICE_PORT}");

    let stream = TcpStream::connect((Ipv4Addr::LOCALHOST, HOST_PORT))?;
    stream.set_read_timeout(Some(TIMEOUT))?;
    (&stream).write_all(b"ping\n")?;
    let answer = read_line(&stream)?;

    let request = requests.recv_timeout(TIMEOUT).map_err(|_| {
        RustADBError::ADBRequestFailed("host server received no connection".into())
    })??;
    check("host server received", &request, "ping")?;
    check("host client received", &answer, "pong")?;

    device.forward_remove(&format!("tcp:{HOST_PORT}"))?;
    device.reverse_remove(&format!("tcp:{DEVICE_PORT}"))?;
    println!("forward and reverse work both ways");
    Ok(())
}

/// Answer `pong` to the first connection, and return what it sent.
fn serve(listener: &TcpListener) -> Result<String> {
    let (stream, _) = listener.accept()?;
    stream.set_read_timeout(Some(TIMEOUT))?;
    let request = read_line(&stream)?;
    (&stream).write_all(b"pong\n")?;
    Ok(request)
}

/// Read a line from `stream`, without its line ending. Empty if the connection is closed first.
fn read_line(stream: &TcpStream) -> Result<String> {
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line)?;
    Ok(line.trim_end().to_string())
}

fn check(what: &str, received: &str, expected: &str) -> Result<()> {
    println!("{what}: {received:?}");
    if received != expected {
        return Err(RustADBError::ADBRequestFailed(format!(
            "{what} {received:?} instead of {expected:?}"
        )));
    }

    Ok(())
}
