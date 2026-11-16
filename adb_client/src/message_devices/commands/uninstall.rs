use crate::{
    Result,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
    },
    models::ADBLocalCommand,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn uninstall(&self, package_name: &str, user: Option<&str>) -> Result<()> {
        let mut session = self.open_session(&ADBLocalCommand::Uninstall(
            package_name.to_string(),
            user.map(ToString::to_string),
        ))?;

        match session.recv_until_close()?.as_slice() {
            b"Success\n" => {
                log::info!("Package {package_name} successfully uninstalled");
                Ok(())
            }
            d => Err(crate::RustADBError::ADBRequestFailed(String::from_utf8(
                d.to_vec(),
            )?)),
        }
    }
}
