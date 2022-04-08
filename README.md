# Rust ADB (Android Debug Bridge) client library

This crate is not affiliated with Android development core team.

It is still under active development, please report found bugs as issues !

## Rust crate

Simply add this to your `Cargo.toml`:
```toml
[dependencies]
adb_client = "*"
```

To launch a command on host device :
```rust
use adb_client::AdbTcpConnexion;
use adb_client::AdbCommandProvider;

let connexion = AdbTcpConnexion::new();
connexion.shell_command("df -h");
```

To get available ADB devices :
```rust
use adb_client::AdbTcpConnexion;
use adb_client::AdbCommandProvider;

let connexion = AdbTcpConnexion::new();
connexion.devices();
```


## Rust binary

You can install the lightweight adb binary by running the following command :
```shell
cargo install adb_client --features adbclient 
```


<https://developer.android.com/studio/command-line/adb>

<https://github.com/cstyan/adbDocumentation>
