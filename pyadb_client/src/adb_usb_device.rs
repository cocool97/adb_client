use adb_client::{ADBDeviceExt, ADBUSBDevice};
use anyhow::Result;
use pyo3::{pyclass, pymethods};

#[pyclass]
pub struct PyADBUSBDevice(ADBUSBDevice);

#[pymethods]
impl PyADBUSBDevice {
    #[staticmethod]
    pub fn autodetect() -> Result<Self> {
        Ok(ADBUSBDevice::autodetect()?.into())
    }

    pub fn shell(&mut self, commands: Vec<String>) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        let commands: Vec<&str> = commands.iter().map(|x| &**x).collect();
        self.0.shell_command(&commands, &mut output)?;
        Ok(output)
    }
}

impl From<ADBUSBDevice> for PyADBUSBDevice {
    fn from(value: ADBUSBDevice) -> Self {
        Self(value)
    }
}
