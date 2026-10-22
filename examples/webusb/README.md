# ADB in the browser over WebUSB

This example connects directly to an Android device over USB, performs ADB RSA
host authentication, and streams legacy shell output. No ADB server or local
proxy is involved. Enable USB debugging on the phone and accept its authorization
dialog. Use a browser with WebUSB support, such as desktop Chrome or Edge, over
HTTPS or localhost. Close other ADB clients before connecting. The operating
system must allow access to the USB interface; on Linux this may require udev
permissions, and on Windows an appropriate USB driver.

## Build and run

From the repository root:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
cargo build -p webusb-example --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir examples/webusb/pkg target/wasm32-unknown-unknown/release/webusb_example.wasm
python3 -m http.server --directory examples/webusb 8000
```

Open <http://localhost:8000>, click **Connect**, select your phone, approve its
debugging dialog, and run a command. **Disconnect / stop** closes USB to interrupt
an outstanding read, then releases the Rust connection. Reloading creates a new
host key and may require another authorization on the phone.

## Using the library

```toml
[dependencies]
adb_client = { version = "3.2.3", default-features = false, features = ["webusb"] }
```

Until this feature is released, use a path dependency as this example does.
Build for `wasm32-unknown-unknown`. The USB bindings in `web-sys` are currently
unstable and require this setting in the application's `.cargo/config.toml`:

```toml
[target.wasm32-unknown-unknown]
rustflags = ["--cfg=web_sys_unstable_apis"]
```

`request_device()` starts the picker synchronously. Call it directly from the
click/key handler **before awaiting any other work**, then await its promise
with `wasm_bindgen_futures::JsFuture`. Pass the selected `web_sys::UsbDevice`
and a `WebUSBKey` to `ADBWebUSBDevice::connect(...).await`.

`WebUSBKey::generate()` uses the browser CSPRNG, but RSA generation itself is
synchronous. Alternatively, generate a 2048-bit RSA key asynchronously with
WebCrypto and import it using `WebUSBKey::from_pkcs8_pem`, as the example does.
The example keeps its PKCS#8 key in memory. Applications can persist and reload
keys with `to_pkcs8_pem` / `from_pkcs8_pem`; treat that private key as a credential.
The library never reads `~/.android`, invokes ADB, or sends the key to a server.

`device.shell_command(command, callback).await` delivers bounded byte chunks.
`device.open(service).await` exposes a service stream with async `read`, `write`,
and `close` methods. `read()` returns `None` when the peer closes the stream;
`write()` chunks data to the peer's limit and waits for acknowledgements.

## Scope and lifecycle

- The browser API is separate from the native blocking `ADBDeviceExt` API.
  Native TCP, libusb, CLI, and Python interfaces keep their existing behavior.
- One service stream may be open at a time. Finish with `close().await` or read
  to EOF before opening another stream. Dropping an unfinished stream, a
  cancelled I/O future, or a protocol/I/O error requires disconnect/reconnect.
- `device.disconnect().await` closes the USB device and releases its interface.
  Dropping the device schedules best-effort cleanup; await explicit disconnect
  when deterministic cleanup matters. WebUSB cannot cancel individual reads;
  applications can close the selected `UsbDevice` to abort a pending operation,
  then dispose of the Rust connection after that operation returns.
- The implementation negotiates checksummed ADB 1.0 with 64 KiB receive packets
  and legacy per-packet acknowledgements. It does not advertise delayed ACKs or
  shell v2. Shell combines stdout/stderr and does not return an exit status.
- Raw services such as `sync:` are available through `open`, but high-level
  filesystem, framebuffer, and other native command helpers are not ported.

## Automated tests

Native protocol tests cover RSA authentication, fragmented packets and USB ZLPs,
payload limits, crossed traffic/backpressure, stream IDs, malformed packets,
and cancelled operations. JavaScript USB mocks exercise descriptor selection,
DataView offsets, transfer errors, ZLP writes, and cleanup after failed or
cancelled interface setup. They run as WebAssembly under Node:

```sh
cargo test -p adb_client --lib --features webusb
cargo test -p adb_client --lib --target wasm32-unknown-unknown --no-default-features --features webusb --config 'target.wasm32-unknown-unknown.runner="wasm-bindgen-test-runner"' webusb::browser::tests
```

These mocks do not verify browser permission prompts, OS USB permissions, or a
physical Android device; those still need hardware testing.
