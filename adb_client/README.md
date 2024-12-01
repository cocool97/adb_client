# adb_client

[![MIT licensed](https://img.shields.io/crates/l/adb_client.svg)](./LICENSE-MIT)
[![Documentation](https://docs.rs/adb_client/badge.svg)](https://docs.rs/adb_client)
![Crates.io Total Downloads](https://img.shields.io/crates/d/adb_client)

Rust library implementing ADB protocol.

## Installation

Add `adb_client` crate as a dependency by simply adding it to your `Cargo.toml`:

```toml
[dependencies]
adb_client = "*"
```

## Examples

### Get available ADB devices

```rust no_run
use adb_client::ADBServer;
use std::net::{SocketAddrV4, Ipv4Addr};

// A custom server address can be provided
let server_ip = Ipv4Addr::new(127, 0, 0, 1);
let server_port = 5037;

let mut server = ADBServer::new(SocketAddrV4::new(server_ip, server_port));
server.devices();
```

### Using ADB server as bridge

#### Launch a command on device

```rust no_run
use adb_client::{ADBServer, ADBDeviceExt};

let mut server = ADBServer::default();
let mut device = server.get_device().expect("cannot get device");
device.shell_command(["df", "-h"],std::io::stdout());
```

#### Push a file to the device

```rust no_run
use adb_client::ADBServer;
use std::net::Ipv4Addr;
use std::fs::File;
use std::path::Path;

let mut server = ADBServer::default();
let mut device = server.get_device().expect("cannot get device");
let mut input = File::open(Path::new("/tmp/f")).expect("Cannot open file");
device.push(&mut input, "/data/local/tmp");
```

### Interact directly with end devices

#### (USB) Launch a command on device

```rust no_run
use adb_client::{ADBUSBDevice, ADBDeviceExt};

let vendor_id = 0x04e8;
let product_id = 0x6860;
let mut device = ADBUSBDevice::new(vendor_id, product_id).expect("cannot find device");
device.shell_command(["df", "-h"],std::io::stdout());
```

#### (USB) Push a file to the device

```rust no_run
use adb_client::{ADBUSBDevice, ADBDeviceExt};
use std::fs::File;
use std::path::Path;

let vendor_id = 0x04e8;
let product_id = 0x6860;
let mut device = ADBUSBDevice::new(vendor_id, product_id).expect("cannot find device");
let mut input = File::open(Path::new("/tmp/f")).expect("Cannot open file");
device.push(&mut input, "/data/local/tmp");
```

### (TCP) Get a shell from device

```rust no_run
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use adb_client::{ADBTcpDevice, ADBDeviceExt};

let device_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 10));
let device_port = 43210;
let mut device = ADBTcpDevice::new(SocketAddr::new(device_ip, device_port)).expect("cannot find device");
device.shell(std::io::stdin(), std::io::stdout());
```
