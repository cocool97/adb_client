#![forbid(missing_docs)]
#![doc = include_str!("../README.md")]

mod adb_server;
mod adb_server_device;
mod adb_usb_device;
mod models;
pub use adb_server::*;
pub use adb_server_device::*;
pub use adb_usb_device::*;
pub use models::*;

use pyo3::prelude::*;
use pyo3_stub_gen::StubInfo;

#[pymodule]
fn pyadb_client(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyADBServer>()?;
    m.add_class::<PyDeviceShort>()?;
    m.add_class::<PyADBServerDevice>()?;
    m.add_class::<PyADBUSBDevice>()?;

    Ok(())
}

/// Get stub informations for this package.
pub fn stub_info() -> anyhow::Result<StubInfo> {
    // Need to be run from workspace root directory
    StubInfo::from_pyproject_toml(format!("{}/pyproject.toml", env!("CARGO_MANIFEST_DIR")))
}
