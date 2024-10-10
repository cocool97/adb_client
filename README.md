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
    </p>
</p>

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

## Related publications

- [Diving into ADB protocol internals (1/2)](https://www.synacktiv.com/publications/diving-into-adb-protocol-internals-12)

All pull requests are welcome !
