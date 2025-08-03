# adb_cli

[![MIT licensed](https://img.shields.io/crates/l/adb_cli.svg)](./LICENSE-MIT)
![Crates.io Total Downloads](https://img.shields.io/crates/d/adb_cli)
![MSRV](https://img.shields.io/crates/msrv/adb_cli)

Rust binary providing an improved version of `adb` CLI.

## Rust binary

This crate provides a lightweight binary based on the `adb_client` crate. You can install it by running the following command :

```shell
cargo install adb_cli
```

Usage is quite simple, and tends to look like `adb`:

- To use ADB server as a proxy:

```bash
user@laptop ~/adb_client (main)> adb_cli local --help
Device related commands using server

Usage: adb_cli local [OPTIONS] <COMMAND>

Commands:
  shell          Spawn an interactive shell or run a list of commands on the device
  pull           Pull a file from device
  push           Push a file on device
  stat           Stat a file on device
  run            Run an activity on device specified by the intent
  reboot         Reboot the device
  install        Install an APK on device
  framebuffer    Dump framebuffer of device
  host-features  List available server features
  list           List a directory on device
  logcat         Get logs of device
  help           Print this message or the help of the given subcommand(s)

Options:
  -a, --address <ADDRESS>  [default: 127.0.0.1:5037]
  -s, --serial <SERIAL>    Serial id of a specific device. Every request will be sent to this device
  -h, --help               Print help
```

- To interact directly with end devices

```bash
user@laptop ~/adb_client (main)> adb_cli usb --help
Device commands via USB, no server needed

Usage: adb_cli usb [OPTIONS] --vendor-id <VID> --product-id <PID> <COMMAND>

Commands:
  shell   Spawn an interactive shell or run a list of commands on the device
  pull    Pull a file from device
  push    Push a file on device
  stat    Stat a file on device
  reboot  Reboot the device
  help    Print this message or the help of the given subcommand(s)

Options:
  -v, --vendor-id <VID>                    Hexadecimal vendor id of this USB device
  -p, --product-id <PID>                   Hexadecimal product id of this USB device
  -k, --private-key <PATH_TO_PRIVATE_KEY>  Path to a custom private key to use for authentication
  -h, --help                               Print help
```
