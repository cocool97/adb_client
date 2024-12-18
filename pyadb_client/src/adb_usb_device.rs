use std::{fs::File, path::PathBuf};

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

    pub fn shell_command(&mut self, commands: Vec<String>) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        let commands: Vec<&str> = commands.iter().map(|x| &**x).collect();
        self.0.shell_command(&commands, &mut output)?;
        Ok(output)
    }

    pub fn push(&mut self, input: PathBuf, dest: PathBuf) -> Result<()> {
        let mut reader = File::open(input)?;
        Ok(self.0.push(&mut reader, &dest.to_string_lossy())?)
    }

    pub fn pull(&mut self, input: PathBuf, dest: PathBuf) -> Result<()> {
        let mut writer = File::create(dest)?;
        Ok(self.0.pull(&input.to_string_lossy(), &mut writer)?)
    }
}

impl From<ADBUSBDevice> for PyADBUSBDevice {
    fn from(value: ADBUSBDevice) -> Self {
        Self(value)
    }
}
