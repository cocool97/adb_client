<p align="center" style="text-align: center">
  <img src="assets/logo.png" width="33%">
</p>

<p align="center">
    <p align="center">Android Debug Bridge (ADB) client implementation in pure Rust !</p>
    <p align="center">
        <a href="https://crates.io/crates/adb_client">
            <img alt="crates.io" src="https://img.shields.io/crates/v/adb_client.svg"/>
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
- Supports:
  - **TCP/IP** protocol, using ADB server as a proxy (standard behavior when using `adb` CLI)
  - **USB** protocol, interacting directly with end devices
- Implements hidden `adb` features, like `framebuffer`
- Highly configurable
- Easy to use !

## adb_client

Rust library implementing both ADB protocols and providing a high-level abstraction over many supported commands.

Improved documentation [here](./adb_client/README.md).

## adb_cli

Rust binary providing an improved version of official `adb` CLI, wrapping `adb_client` library. Can act as an usage example of the library.

Improved documentation [here](./adb_cli/README.md).

## Related publications

- [Diving into ADB protocol internals (1/2)](https://www.synacktiv.com/publications/diving-into-adb-protocol-internals-12)

Some features may still be missing, all pull requests are welcome !
