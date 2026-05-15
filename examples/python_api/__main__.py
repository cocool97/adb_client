import logging
import random
from importlib.metadata import version
from tempfile import NamedTemporaryFile

from pyadb_client import PyADBUSBDevice

logger = logging.getLogger(__name__)
logging.basicConfig(
    format="%(asctime)s - %(levelname)s - %(message)s",
    level=logging.DEBUG,
)

RANDOM_DATA_SIZE = 1 * 1024 * 1024  # 1 MB


def main():
    logger.info(f"running pyadb_client version: {version('pyadb_client')}")
    # Detect the device automatically
    # Only one device must be connected using this method.
    device = PyADBUSBDevice.autodetect()

    device_vendor_id = device.vendor_id()
    device_product_id = device.product_id()

    logger.info("found device")
    logger.info(
        f"vendor_id={hex(device_vendor_id)}, product_id={hex(device_product_id)}"
    )

    # Generate a random data into a temporary file to push on the device
    data = random.randbytes(RANDOM_DATA_SIZE)
    with NamedTemporaryFile() as f:
        f.write(data)
        f.seek(0)

        device.push(f.name, "/data/local/tmp/random_data")

    # Get the data back from the device and verify if it matches
    with NamedTemporaryFile() as f:
        device.pull("/data/local/tmp/random_data", f.name)
        f.seek(0)
        assert f.read() == data
        logger.info("pulled data matches")

    uid = int(device.shell_command("id -u"))
    logger.info(f"uid={uid}")


if __name__ == "__main__":
    main()
