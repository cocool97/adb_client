#!/usr/bin/env bash

ADB_CLIENT_CRATE_NAME="adb_client"
ADB_SERVER_DEVICE_FULL_TESTSUITE_NAME="full_server_device"

echo "[ ] Running ADB server device test suite..."
cargo test --release -p ${ADB_CLIENT_CRATE_NAME} --test ${ADB_SERVER_DEVICE_FULL_TESTSUITE_NAME} -- --ignored
echo "[X] ADB server device test suite finished."
