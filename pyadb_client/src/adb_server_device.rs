use adb_client::{ADBDeviceExt, Connected, server_device::ADBServerDevice};
use anyhow::Result;
use pyo3::{pyclass, pymethods};
use pyo3_stub_gen_derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::{fs::File, path::PathBuf};

#[gen_stub_pyclass]
#[pyclass]
/// Represent a device connected to the ADB server
pub struct PyADBServerDevice(pub ADBServerDevice<Connected>);

#[gen_stub_pymethods]
#[pymethods]
impl PyADBServerDevice {
    #[must_use]
    #[getter]
    /// Device identifier
    pub fn identifier(&self) -> Option<String> {
        self.0.identifier.clone()
    }

    /// Run shell commands on device and return the output (stdout + stderr merged)
    pub fn shell_command(&mut self, command: &str) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        self.0.shell_command(&command, &mut output)?;
        Ok(output)
    }

    /// Push a local file from input to dest
    #[expect(clippy::needless_pass_by_value)]
    pub fn push(&mut self, input: PathBuf, dest: PathBuf) -> Result<()> {
        let mut reader = File::open(input)?;
        Ok(self.0.push(&mut reader, dest.to_string_lossy())?)
    }

    /// Pull a file from device located at input, and drop it to dest
    #[expect(clippy::needless_pass_by_value)]
    pub fn pull(&mut self, input: PathBuf, dest: PathBuf) -> Result<()> {
        let mut writer = File::create(dest)?;
        Ok(self.0.pull(&input.to_string_lossy(), &mut writer)?)
    }

    /// Install a package installed on the device
    #[expect(clippy::needless_pass_by_value)]
    pub fn install(&mut self, apk_path: PathBuf) -> Result<()> {
        Ok(self.0.install(&apk_path)?)
    }

    /// Uninstall a package installed on the device
    pub fn uninstall(&mut self, package: &str) -> Result<()> {
        Ok(self.0.uninstall(package)?)
    }
}

impl From<ADBServerDevice<Connected>> for PyADBServerDevice {
    fn from(value: ADBServerDevice<Connected>) -> Self {
        Self(value)
    }
}
