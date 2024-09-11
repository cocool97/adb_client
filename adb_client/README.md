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

### Launch a command on device via ADB server

```rust no_run
use adb_client::ADBServer;

let mut server = ADBServer::default();
let mut device = server.get_device().expect("cannot get device");
device.shell_command(["df", "-h"],std::io::stdout());
```

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

### Push a file to the device

```rust no_run
use adb_client::ADBServer;
use std::net::Ipv4Addr;
use std::fs::File;
use std::path::Path;

let mut server = ADBServer::default();
let mut device = server.get_device().expect("cannot get device");
let mut input = File::open(Path::new("/tmp")).unwrap();
device.send(&mut input, "/data/local/tmp");
```
