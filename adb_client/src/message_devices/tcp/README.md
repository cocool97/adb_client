# Examples

> TCP-connected devices related examples

## Get a shell from device

```rust no_run
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use adb_client::{tcp::ADBTcpDevice, ADBDeviceExt};

let device_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 10));
let device_port = 43210;
let mut device = ADBTcpDevice::new(SocketAddr::new(device_ip, device_port)).expect("cannot find device");
device.shell(&mut std::io::stdin(), Box::new(std::io::stdout()));
```
