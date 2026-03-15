use std::{fs::File, path::PathBuf};

use adb_client::{ADBDeviceExt, tcp::ADBTcpDevice};
use anyhow::Result;
use pyo3::{pyclass, pymethods};
use pyo3_stub_gen_derive::{gen_stub_pyclass, gen_stub_pymethods};

#[gen_stub_pyclass]
#[pyclass]
/// Represent a device directly reachable over TCP.
pub struct PyADBTcpDevice(ADBTcpDevice);

#[gen_stub_pymethods]
#[pymethods]
impl PyADBTcpDevice {
    #[staticmethod]
    /// Create a new TCP device connection with the given address (e.g. "192.168.1.100:5555").
    pub fn new(address: &str) -> Result<Self> {
        let addr: std::net::SocketAddr = address.parse()?;
        Ok(ADBTcpDevice::new(addr)?.into())
    }

    #[staticmethod]
    /// Create a new TCP device connection with the given address, using a custom private key.
    pub fn new_with_custom_private_key(address: &str, private_key_path: PathBuf) -> Result<Self> {
        let addr: std::net::SocketAddr = address.parse()?;
        Ok(ADBTcpDevice::new_with_custom_private_key(addr, private_key_path)?.into())
    }

    /// Run shell commands on device and return the output (stdout + stderr merged)
    pub fn shell_command(&mut self, command: &str) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        self.0.shell_command(&command, Some(&mut output), None)?;
        Ok(output)
    }

    /// Push a local file from input to dest
    #[expect(clippy::needless_pass_by_value)]
    pub fn push(&mut self, input: PathBuf, dest: PathBuf) -> Result<()> {
        let mut reader = File::open(input)?;
        Ok(self.0.push(&mut reader, &dest.to_string_lossy())?)
    }

    /// Pull a file from device located at input, and drop it to dest
    #[expect(clippy::needless_pass_by_value)]
    pub fn pull(&mut self, input: PathBuf, dest: PathBuf) -> Result<()> {
        let mut writer = File::create(dest)?;
        Ok(self.0.pull(&input.to_string_lossy(), &mut writer)?)
    }

    /// Install a package (APK) onto the device from the given local path
    #[expect(clippy::needless_pass_by_value)]
    #[pyo3(signature = (apk_path, user=None))]
    pub fn install(&mut self, apk_path: PathBuf, user: Option<&str>) -> Result<()> {
        Ok(self.0.install(&apk_path, user)?)
    }

    /// Uninstall a package installed on the device
    #[pyo3(signature = (package, user=None))]
    pub fn uninstall(&mut self, package: &str, user: Option<&str>) -> Result<()> {
        Ok(self.0.uninstall(&package, user)?)
    }

    /// Restart adb daemon with root permissions
    pub fn root(&mut self) -> Result<()> {
        Ok(self.0.root()?)
    }
}

impl From<ADBTcpDevice> for PyADBTcpDevice {
    fn from(value: ADBTcpDevice) -> Self {
        Self(value)
    }
}
