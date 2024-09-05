# adb_cli

[![MIT licensed](https://img.shields.io/crates/l/adb_cli.svg)](./LICENSE-MIT)
![Crates.io Total Downloads](https://img.shields.io/crates/d/adb_cli)

Rust binary providing an improved version of `adb` CLI.

## Rust binary

This crate provides a lightweight binary based on the `adb_client` crate. You can install it by running the following command :

```shell
cargo install adb_cli 
```

Usage is quite simple, and tends to look like `adb`:

```bash
user@laptop ~/adb_client (main)> adb_cli --help
Rust ADB (Android Debug Bridge) CLI

Usage: adb_cli [OPTIONS] <COMMAND>

Commands:
  host-features  List available server features
  push           Push a file on device
  pull           Pull a file from device
  list           List a directory on device
  stat           Stat a file specified on device
  shell          Spawn an interactive shell or run a list of commands on the device
  reboot         Reboot the device
  framebuffer    Dump framebuffer of device
  logcat         Get logs of device
  version        Print current ADB version
  kill           Ask ADB server to quit immediately
  devices        List connected devices
  track-devices  Track new devices showing up
  pair           Pair device with a given code
  connect        Connect device over WI-FI
  disconnect     Disconnect device over WI-FI
  sms            Send a SMS with given phone number and given content
  rotate         Rotate device screen from 90°
  help           Print this message or the help of the given subcommand(s)

Options:
  -d, --debug              
  -a, --address <ADDRESS>  [default: 127.0.0.1:5037]
  -s, --serial <SERIAL>    Serial id of a specific device. Every request will be sent to this device
  -h, --help               Print help
  -V, --version            Print version
```
