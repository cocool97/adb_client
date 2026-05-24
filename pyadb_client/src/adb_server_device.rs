use adb_client::{ADBDeviceExt, server_device::ADBServerDevice};
use anyhow::Result;
use pyo3::{Bound, Python, pyclass, pymethods, types::PyBytes};
use pyo3_stub_gen_derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::{fs::File, path::PathBuf};

#[gen_stub_pyclass]
#[pyclass]
/// Represent a device connected to the ADB server
pub struct PyADBServerDevice(pub ADBServerDevice);

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
    pub fn shell_command<'py>(
        &mut self,
        py: Python<'py>,
        command: &str,
    ) -> Result<Bound<'py, PyBytes>> {
        let mut output = Vec::new();
        self.0.shell_command(command, Some(&mut output), None)?;
        Ok(PyBytes::new(py, &output))
    }

    /// Push a local file from input to dest
    pub fn push(&mut self, input: PathBuf, dest: PathBuf) -> Result<()> {
        let mut reader = File::open(input)?;
        Ok(self.0.push(&mut reader, dest)?)
    }

    /// Pull a file from device located at input, and drop it to dest
    pub fn pull(&mut self, input: PathBuf, dest: PathBuf) -> Result<()> {
        let mut writer = File::create(dest)?;
        Ok(self.0.pull(input, &mut writer)?)
    }

    /// Install a package installed on the device
    #[expect(clippy::needless_pass_by_value)]
    #[pyo3(signature = (apk_path, user=None))]
    pub fn install(&mut self, apk_path: PathBuf, user: Option<&str>) -> Result<()> {
        Ok(self.0.install(&apk_path, user)?)
    }

    /// Uninstall a package installed on the device
    #[pyo3(signature = (package, user=None))]
    pub fn uninstall(&mut self, package: &str, user: Option<&str>) -> Result<()> {
        Ok(self.0.uninstall(package, user)?)
    }

    /// Restart adb daemon with root permissions
    pub fn root(&mut self) -> Result<()> {
        Ok(self.0.root()?)
    }
}

impl From<ADBServerDevice> for PyADBServerDevice {
    fn from(value: ADBServerDevice) -> Self {
        Self(value)
    }
}
