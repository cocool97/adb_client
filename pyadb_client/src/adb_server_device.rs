use adb_client::ADBServerDevice;
use pyo3::{pyclass, pymethods};

#[pyclass]
pub struct PyADBServerDevice(pub ADBServerDevice);

#[pymethods]
impl PyADBServerDevice {
    #[getter]
    pub fn identifier(&self) -> String {
        self.0.identifier.clone()
    }
}

impl From<ADBServerDevice> for PyADBServerDevice {
    fn from(value: ADBServerDevice) -> Self {
        Self(value)
    }
}
