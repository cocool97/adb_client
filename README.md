# adb_client

Android Debug Bridge (ADB) client implementation in pure Rust !

Main features :

- Full Rust, no need to use shell commands
- Currently only support server TCP/IP protocol
- Highly configurable
- Easy to use !

## Examples

First declare `adb_client` as a dependency by simply adding this to your `Cargo.toml`:

```toml
[dependencies]
adb_client = "*"
```

### Launch a command on host device via ADB server

```rust
use adb_client::ADBServer;

let mut server = ADBServer::default();
let mut device = server.get_device(None).expect("cannot get device");
device.shell_command(["df", "-h"]);
```

### Get available ADB devices

```rust
use adb_client::ADBServer;
use std::net::{SocketAddrV4, Ipv4Addr};

let server_ip = Ipv4Addr::new(127, 0, 0, 1);
let server_port = 5037;
let mut server = ADBServer::new(SocketAddrV4::new(server_ip, server_port));
server.devices();
```

### Push a file to the device

```rust
use adb_client::ADBServer;
use std::net::Ipv4Addr;
use std::fs::File;
use std::path::Path;

let mut server = ADBServer::default();
let mut device = server.get_device(None).expect("cannot get device");
let mut input = File::open(Path::new("/tmp")).unwrap();
device.send(&mut input, "/data/local/tmp");
```

## Rust binary

This crate also provides a lightweight binary based on the `adb_client` crate. You can install it by running the following command :

```shell
cargo install adb_client --example adb_cli 
```

## Missing features

- USB protocol

All pull requests are welcome !

## Documentation

- <https://developer.android.com/studio/command-line/adb>

- <https://github.com/cstyan/adbDocumentation>
