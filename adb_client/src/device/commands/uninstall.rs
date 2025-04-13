use crate::{ADBMessageTransport, Result, device::adb_message_device::ADBMessageDevice};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn uninstall(&mut self, package_name: &str) -> Result<()> {
        self.open_session(format!("exec:cmd package 'uninstall' {}\0", package_name).as_bytes())?;

        let final_status = self.get_transport_mut().read_message()?;

        match final_status.into_payload().as_slice() {
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
