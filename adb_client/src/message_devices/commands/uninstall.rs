use crate::{
    Result,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
    },
    models::ADBLocalCommand,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn uninstall(
        &mut self,
        package_name: &dyn AsRef<str>,
        user: Option<&str>,
    ) -> Result<()> {
        let mut session = self.open_session(&ADBLocalCommand::Uninstall(
            package_name.as_ref().to_string(),
            user.map(ToString::to_string),
        ))?;

        let final_status = session.read_message()?;

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
