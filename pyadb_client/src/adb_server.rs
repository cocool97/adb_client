use std::net::SocketAddrV4;

use adb_client::ADBServer;
use anyhow::Result;
use pyo3::{PyResult, pyclass, pymethods};
use pyo3_stub_gen_derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::{PyADBServerDevice, PyDeviceShort};

#[gen_stub_pyclass]
#[pyclass]
/// Represent an instance of an ADB Server
pub struct PyADBServer(ADBServer);

#[gen_stub_pymethods]
#[pymethods]
impl PyADBServer {
    #[new]
    /// Instantiate a new PyADBServer instance
    pub fn new(address: String) -> PyResult<Self> {
        let address = address.parse::<SocketAddrV4>()?;
        Ok(ADBServer::new(address).into())
    }

    /// List available devices
    pub fn devices(&mut self) -> Result<Vec<PyDeviceShort>> {
        Ok(self.0.devices()?.into_iter().map(|v| v.into()).collect())
    }

    /// Get a device, assuming that only one is currently connected
    pub fn get_device(&mut self) -> Result<PyADBServerDevice> {
        Ok(self.0.get_device()?.into())
    }

    /// Get a device by its name, as shown in `.devices()` output
    pub fn get_device_by_name(&mut self, name: String) -> Result<PyADBServerDevice> {
        Ok(self.0.get_device_by_name(&name)?.into())
    }
}

impl From<ADBServer> for PyADBServer {
    fn from(value: ADBServer) -> Self {
        Self(value)
    }
}
