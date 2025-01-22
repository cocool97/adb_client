use std::net::SocketAddrV4;

use adb_client::ADBServer;
use anyhow::Result;
use pyo3::{pyclass, pymethods, PyResult};

use crate::{PyADBServerDevice, PyDeviceShort};

#[pyclass]
pub struct PyADBServer(ADBServer);

#[pymethods]
impl PyADBServer {
    #[new]
    pub fn new(address: String) -> PyResult<Self> {
        let address = address.parse::<SocketAddrV4>()?;
        Ok(ADBServer::new(address).into())
    }

    pub fn devices(&mut self) -> Result<Vec<PyDeviceShort>> {
        Ok(self.0.devices()?.into_iter().map(|v| v.into()).collect())
    }

    pub fn get_device(&mut self) -> Result<PyADBServerDevice> {
        Ok(self.0.get_device()?.into())
    }

    pub fn get_device_by_name(&mut self, name: String) -> Result<PyADBServerDevice> {
        Ok(self.0.get_device_by_name(&name)?.into())
    }
}

impl From<ADBServer> for PyADBServer {
    fn from(value: ADBServer) -> Self {
        Self(value)
    }
}
