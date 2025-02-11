use adb_client::DeviceShort;
use pyo3::{pyclass, pymethods};
use pyo3_stub_gen_derive::{gen_stub_pyclass, gen_stub_pymethods};

// Check https://docs.rs/rigetti-pyo3/latest/rigetti_pyo3 to automatically build this code

#[gen_stub_pyclass]
#[pyclass]
/// Represent a device output as shown when running `adb devices`
pub struct PyDeviceShort(DeviceShort);

#[gen_stub_pymethods]
#[pymethods]
impl PyDeviceShort {
    #[getter]
    /// Device identifier
    pub fn identifier(&self) -> String {
        self.0.identifier.clone()
    }

    #[getter]
    /// Device state
    pub fn state(&self) -> String {
        self.0.state.to_string()
    }
}

impl From<DeviceShort> for PyDeviceShort {
    fn from(value: DeviceShort) -> Self {
        Self(value)
    }
}
