<p align="center" style="text-align: center">
  <img src="assets/logo.png" width="33%">
</p>

<p align="center">
    <p align="center">Android Debug Bridge (ADB) client implementation in pure Rust !</p>
    <p align="center">
        <a href="https://crates.io/crates/adb_client">
            <img alt="crates.io" src="https://img.shields.io/crates/v/adb_client.svg"/>
        </a>
        <a href="https://crates.io/crates/adb_client">
            <img alt="msrv" src="https://img.shields.io/crates/msrv/adb_client"/>
        </a>
        <a href="https://github.com/cocool97/adb_client/actions">
            <img alt="ci status" src="https://github.com/cocool97/adb_client/actions/workflows/rust-build-matrix.yml/badge.svg"/>
        </a>
        <a href="https://deps.rs/repo/github/cocool97/adb_client">
            <img alt="dependency status" src="https://deps.rs/repo/github/cocool97/adb_client/status.svg"/>
        </a>
        <a href="https://opensource.org/licenses/MIT">
            <img alt="dependency status" src="https://img.shields.io/badge/License-MIT-yellow.svg"/>
        </a>
    </p>
</p>

Main features of this library:

- Full Rust, don't use `adb *` shell commands to interact with devices
- Supports
  - Using ADB server as a proxy (standard behavior when using `adb` CLI)
  - Connecting directly to end devices (without using adb-server)
    - Over **USB**
    - Over **TCP/IP**
- Implements hidden `adb` features, like `framebuffer`
- Highly configurable
- Provides wrappers to use directly from Python code
- Easy to use !

## adb_client

Rust library implementing both ADB protocols (server and end-devices) and providing a high-level abstraction over the many supported commands.

Improved documentation available [here](./adb_client/README.md).

## examples

Some examples showing of to use this library are available in the `examples` directory:

- `examples/mdns`: mDNS device discovery

## adb_cli

Rust binary providing an improved version of Google's official `adb` CLI, by using `adb_client` library.
Provides a "real-world" usage example of this library.

Improved documentation available [here](./adb_cli/README.md).

## examples

Some examples are available in the `examples` directory:

- `examples/mdns`: mDNS device discovery example

## pyadb_client

Python wrapper using `adb_client` library to export classes usable directly from a Python environment.

Improved documentation available [here](./pyadb_client/README.md)

## Related publications

- [Diving into ADB protocol internals (1/2)](https://www.synacktiv.com/publications/diving-into-adb-protocol-internals-12)
- [Diving into ADB protocol internals (2/2)](https://www.synacktiv.com/publications/diving-into-adb-protocol-internals-22)

Some features may still be missing, all pull requests are welcome !
