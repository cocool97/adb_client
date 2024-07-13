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

### Launch a command on host device

```rust
use adb_client::AdbTcpConnection;
use std::net::Ipv4Addr;

let mut connection = AdbTcpConnection::new(Ipv4Addr::from([127,0,0,1]), 5037).unwrap();
connection.shell_command(None, ["df", "-h"]);
```

### Get available ADB devices

```rust
use adb_client::AdbTcpConnection;
use std::net::Ipv4Addr;

let mut connection = AdbTcpConnection::new(Ipv4Addr::from([127,0,0,1]), 5037).unwrap();
connection.devices();
```

### Push a file to the device

```rust
use adb_client::AdbTcpConnection;
use std::net::Ipv4Addr;
use std::fs::File;
use std::path::Path;

let mut connection = AdbTcpConnection::new(Ipv4Addr::from([127,0,0,1]), 5037).unwrap();
let serial: Option<&str> = None;
let mut input = File::open(Path::new("/tmp")).unwrap();
connection.send(serial, &mut input, "/data/local/tmp");
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
