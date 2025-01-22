# pyadb_client

Python library to communicate with ADB devices. Built on top of Rust `adb_client` library.

## Examples

### Use ADB server

```python
server = pyadb_client.PyADBServer("127.0.0.1:5037")
for i, device in enumerate(server.devices()):
    print(i, device.identifier, device.state)

# Get only connected device
device = server.get_device()
print(device, device.identifier)
```

### Push a file on device

```python
usb_device = PyADBUSBDevice.autodetect()
usb_device.push("file.txt", "/data/local/tmp/file.txt")
```

## Local development

```bash
# Create Python virtual environment
cd pyadb_client
python3 -m venv .venv
source .venv/bin/activate

# Install needed dependencies
pip install -e .

# Build development package
maturin develop

# Build release Python package
maturin build --release

# Publish Python package
```