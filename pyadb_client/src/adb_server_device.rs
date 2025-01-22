use adb_client::{ADBDeviceExt, ADBServerDevice};
use anyhow::Result;
use pyo3::{pyclass, pymethods};
use std::{fs::File, path::PathBuf};

#[pyclass]
pub struct PyADBServerDevice(pub ADBServerDevice);

#[pymethods]
impl PyADBServerDevice {
    #[getter]
    pub fn identifier(&self) -> String {
        self.0.identifier.clone()
    }

    pub fn shell_command(&mut self, commands: Vec<String>) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        let commands: Vec<&str> = commands.iter().map(|x| &**x).collect();
        self.0.shell_command(&commands, &mut output)?;
        Ok(output)
    }

    pub fn push(&mut self, input: PathBuf, dest: PathBuf) -> Result<()> {
        let mut reader = File::open(input)?;
        Ok(self.0.push(&mut reader, dest.to_string_lossy())?)
    }

    pub fn pull(&mut self, input: PathBuf, dest: PathBuf) -> Result<()> {
        let mut writer = File::create(dest)?;
        Ok(self.0.pull(&input.to_string_lossy(), &mut writer)?)
    }
}

impl From<ADBServerDevice> for PyADBServerDevice {
    fn from(value: ADBServerDevice) -> Self {
        Self(value)
    }
}
