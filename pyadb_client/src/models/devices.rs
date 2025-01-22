use adb_client::DeviceShort;
use pyo3::{pyclass, pymethods};

// Check https://docs.rs/rigetti-pyo3/latest/rigetti_pyo3 to automatically build this code

#[pyclass]
pub struct PyDeviceShort(DeviceShort);

#[pymethods]
impl PyDeviceShort {
    #[getter]
    pub fn identifier(&self) -> String {
        self.0.identifier.clone()
    }

    #[getter]
    pub fn state(&self) -> String {
        self.0.state.to_string()
    }
}

impl From<DeviceShort> for PyDeviceShort {
    fn from(value: DeviceShort) -> Self {
        Self(value)
    }
}
