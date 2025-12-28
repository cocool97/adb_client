use crate::{
    Result,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
    },
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn uninstall(&mut self, package_name: &dyn AsRef<str>) -> Result<()> {
        self.open_session(
            format!(
                "exec:cmd package 'uninstall' {}{}",
                package_name.as_ref(),
                "\0"
            )
            .as_bytes(),
        )?;

        let final_status = self.get_transport_mut().read_message()?;

        match final_status.into_payload().as_slice() {
            b"Success\n" => {
                log::info!("Package {} successfully uninstalled", package_name.as_ref());
                Ok(())
            }
            d => Err(crate::RustADBError::ADBRequestFailed(String::from_utf8(
                d.to_vec(),
            )?)),
        }
    }
}
