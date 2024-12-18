mod adb_server;
mod adb_server_device;
mod adb_usb_device;
mod models;
pub use adb_server::*;
pub use adb_server_device::*;
pub use adb_usb_device::*;
pub use models::*;

use pyo3::prelude::*;

#[pymodule]
fn pyadb_client(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyADBServer>()?;
    m.add_class::<PyDeviceShort>()?;
    m.add_class::<PyADBServerDevice>()?;
    m.add_class::<PyADBUSBDevice>()?;

    Ok(())
}
