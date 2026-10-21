#!/usr/bin/env bash

ADB_CLIENT_CRATE_NAME="adb_client"
ADB_SERVER_DEVICE_FULL_TESTSUITE_NAME="full_server_device"
ADB_USB_DEVICE_FULL_TESTSUITE_NAME="full_usb_device"

# APK used by installation tests, pinned so that every run installs the same one
TEST_APK_PATH="${ADB_CLIENT_CRATE_NAME}/tests/assets/fossify_calculator.apk"
TEST_APK_URL="https://github.com/FossifyOrg/Calculator/releases/download/1.4.0/calculator-10-foss-release.apk"
TEST_APK_SHA256="7c5443c22d6d62c67dfe80df33ea9161ff0fa440f92590bcb45d7008b57613ed"

if [ ! -f "${TEST_APK_PATH}" ]; then
    echo "[ ] Downloading test APK..."
    mkdir -p "$(dirname "${TEST_APK_PATH}")"
    curl -fL -o "${TEST_APK_PATH}" "${TEST_APK_URL}" || { rm -f "${TEST_APK_PATH}"; exit 1; }
    echo "[X] Test APK downloaded."
fi
if ! echo "${TEST_APK_SHA256}  ${TEST_APK_PATH}" | sha256sum --check --quiet; then
    echo "Test APK ${TEST_APK_PATH} is corrupted, remove it to download it again"
    exit 1
fi

# Every suite runs even if a previous one failed, the script fails if any did
STATUS=0

echo "[ ] Running ADB server device test suite..."
cargo test --release -p ${ADB_CLIENT_CRATE_NAME} --test ${ADB_SERVER_DEVICE_FULL_TESTSUITE_NAME} -- --ignored || STATUS=1
echo "[X] ADB server device test suite finished."

# The ADB server claims the USB interface, it must be stopped to reach the device directly
echo "[ ] Stopping ADB server..."
adb kill-server
echo "[X] ADB server stopped."

echo "[ ] Running ADB USB device test suite..."
cargo test --release -p ${ADB_CLIENT_CRATE_NAME} --features usb --test ${ADB_USB_DEVICE_FULL_TESTSUITE_NAME} -- --ignored || STATUS=1
echo "[X] ADB USB device test suite finished."

exit ${STATUS}
