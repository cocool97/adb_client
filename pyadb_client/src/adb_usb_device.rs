use std::{fs::File, path::PathBuf};

use adb_client::{ADBDeviceExt, ADBUSBDevice};
use anyhow::Result;
use pyo3::{pyclass, pymethods};
use pyo3_stub_gen_derive::{gen_stub_pyclass, gen_stub_pymethods};

#[gen_stub_pyclass]
#[pyclass]
/// Represent a device directly reachable over USB.
pub struct PyADBUSBDevice(ADBUSBDevice);

#[gen_stub_pymethods]
#[pymethods]
impl PyADBUSBDevice {
    #[staticmethod]
    /// Autodetect a device reachable over USB.
    /// This method raises an error if multiple devices or none are connected.
    pub fn autodetect() -> Result<Self> {
        Ok(ADBUSBDevice::autodetect()?.into())
    }

    /// Run shell commands on device and return the output (stdout + stderr merged)
    pub fn shell_command(&mut self, commands: Vec<String>) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        let commands: Vec<&str> = commands.iter().map(|x| &**x).collect();
        self.0.shell_command(&commands, &mut output)?;
        Ok(output)
    }

    /// Push a local file from input to dest
    pub fn push(&mut self, input: PathBuf, dest: PathBuf) -> Result<()> {
        let mut reader = File::open(input)?;
        Ok(self.0.push(&mut reader, &dest.to_string_lossy())?)
    }

    /// Pull a file from device located at input, and drop it to dest
    pub fn pull(&mut self, input: PathBuf, dest: PathBuf) -> Result<()> {
        let mut writer = File::create(dest)?;
        Ok(self.0.pull(&input.to_string_lossy(), &mut writer)?)
    }

    /// Install a package installed on the device
    pub fn install(&mut self, apk_path: PathBuf) -> Result<()> {
        Ok(self.0.install(&apk_path)?)
    }

    /// Uninstall a package installed on the device
    pub fn uninstall(&mut self, package: &str) -> Result<()> {
        Ok(self.0.uninstall(package)?)
    }
}

impl From<ADBUSBDevice> for PyADBUSBDevice {
    fn from(value: ADBUSBDevice) -> Self {
        Self(value)
    }
}
