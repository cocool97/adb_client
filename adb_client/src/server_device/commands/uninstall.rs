use std::io::Read;

use crate::{Result, models::AdbServerCommand, server_device::ADBServerDevice};

impl ADBServerDevice {
    /// Uninstall a package from device
    pub fn uninstall(&mut self, package_name: &str) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .send_adb_request(AdbServerCommand::Uninstall(package_name.to_string()))?;

        let mut data = [0; 1024];
        let read_amount = self.transport.get_raw_connection()?.read(&mut data)?;

        match &data[0..read_amount] {
            b"Success\n" => {
                log::info!("Package {} successfully uninstalled", package_name);
                Ok(())
            }
            d => Err(crate::RustADBError::ADBRequestFailed(String::from_utf8(
                d.to_vec(),
            )?)),
        }
    }
}
