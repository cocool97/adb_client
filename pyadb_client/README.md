# pyadb_client

Python library to communicate with ADB devices. Built on top of Rust `adb_client` library.

## Examples

### Push a file on device

```python
usb_device = PyADBUSBDevice.autodetect()
usb_device.push("file.txt", "/data/local/tmp/file.txt")
```

## Development

```bash
# Python virtual environment
cd pyadb_client
python3 -m venv .venv
source .venv/bin/activate

# Install needed dependencies
pip install -e .

# Build development package
maturin develop

# Build release Python package
maturin build --release
```