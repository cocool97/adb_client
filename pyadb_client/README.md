# pyadb_client

Python library to communicate with ADB devices. Built on top of Rust `adb_client` library.

## Installation

```bash
pip install pyadb_client
```

## Examples

### Use ADB server

```python
from pyadb_client import PyADBServer

server = PyADBServer("127.0.0.1:5037")
for i, device in enumerate(server.devices()):
    print(i, device.identifier, device.state)

# Get only connected device
device = server.get_device()
print(device, device.identifier)
```

### Push a file on device

```python
from pyadb_client import PyADBUSBDevice

usb_device = PyADBUSBDevice.autodetect()
usb_device.push("file.txt", "/data/local/tmp/file.txt")
```

## Local development

```bash
# Create Python virtual environment
cd pyadb_client
python3 -m venv .venv
source .venv/bin/activate

# Install needed build dependencies
pip install maturin

# Build development package
maturin develop

# Build stub file (.pyi)
cargo run --bin stub_gen

# Build release Python package
maturin build --release -m pyadb_client/Cargo.toml
```
