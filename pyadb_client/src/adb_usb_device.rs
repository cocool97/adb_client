use std::{fs::File, path::PathBuf};

use adb_client::{ADBDeviceExt, usb::ADBUSBDevice};
use anyhow::Result;
use pyo3::{Bound, Python, pyclass, pymethods, types::PyBytes};
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

    #[staticmethod]
    /// Autodetect a device reachable over USB using a custom private key.
    /// This method raises an error if multiple devices or none are connected.
    pub fn autodetect_with_custom_private_key(private_key_path: PathBuf) -> Result<Self> {
        Ok(ADBUSBDevice::autodetect_with_custom_private_key(private_key_path)?.into())
    }

    #[staticmethod]
    /// Create a new USB device connection with the given vendor and product IDs.
    pub fn new(vendor_id: u16, product_id: u16) -> Result<Self> {
        Ok(ADBUSBDevice::new(vendor_id, product_id)?.into())
    }

    #[staticmethod]
    /// Create a new USB device connection with the given vendor and product IDs, using a custom private key.
    pub fn new_with_custom_private_key(
        vendor_id: u16,
        product_id: u16,
        private_key_path: PathBuf,
    ) -> Result<Self> {
        Ok(
            ADBUSBDevice::new_with_custom_private_key(vendor_id, product_id, private_key_path)?
                .into(),
        )
    }

    /// Returns the vendor ID of the USB device.
    #[must_use]
    pub fn vendor_id(&self) -> u16 {
        self.0.vendor_id()
    }

    /// Returns the product ID of the USB device.
    #[must_use]
    pub fn product_id(&self) -> u16 {
        self.0.product_id()
    }

    /// Run shell commands on device and return the output (stdout + stderr merged)
    pub fn shell_command<'py>(
        &mut self,
        py: Python<'py>,
        command: &str,
    ) -> Result<Bound<'py, PyBytes>> {
        let mut output = Vec::new();
        self.0.shell_command(&command, Some(&mut output), None)?;
        Ok(PyBytes::new(py, &output))
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

    /// Install a package installed on the device
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

impl From<ADBUSBDevice> for PyADBUSBDevice {
    fn from(value: ADBUSBDevice) -> Self {
        Self(value)
    }
}
