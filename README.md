# Rust adb_client

[![Latest version](https://img.shields.io/crates/v/adb_client.svg)](https://crates.io/crates/adb_client)
[![dependency status](https://deps.rs/repo/github/cocool97/adb_client/status.svg)](https://deps.rs/repo/github/cocool97/adb_client)

**A**ndroid **D**ebug **B**ridge (ADB) client implementation in pure Rust !

Main features :

- Full Rust, no need to use `adb *` shell commands
- Currently only support server TCP/IP protocol
- Highly configurable
- Easy to use !

## adb_client

Rust library implementing ADB protocol and providing high-level abstraction over commands.

Improved documentation [here](./adb_client/README.md).

## adb_cli

Rust binary providing an improved version of `adb` CLI, using `adb_client` library. Can be used as an usage example of the library.

Improved documentation [here](./adb_cli/README.md).

## Missing features

- USB protocol (Work in progress)

All pull requests are welcome !
