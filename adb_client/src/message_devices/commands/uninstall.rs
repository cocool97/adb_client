use crate::{
    Result, RustADBError,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
    },
    models::ADBLocalCommand,
    utils::{adb_command_output_is_success, adb_request_uses_missing_cmd, shell_quote},
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn uninstall(
        &mut self,
        package_name: &dyn AsRef<str>,
        user: Option<&str>,
    ) -> Result<()> {
        self.open_session(&ADBLocalCommand::Uninstall(
            package_name.as_ref().to_string(),
            user.map(ToString::to_string),
        ))?;

        let final_status = self.get_transport_mut().read_message()?;

        let uninstall_result = match final_status.into_payload().as_slice() {
            b"Success\n" => {
                log::info!("Package {} successfully uninstalled", package_name.as_ref());
                Ok(())
            }
            d => Err(crate::RustADBError::ADBRequestFailed(String::from_utf8(
                d.to_vec(),
            )?)),
        };

        match uninstall_result {
            Ok(()) => Ok(()),
            Err(RustADBError::ADBRequestFailed(message))
                if adb_request_uses_missing_cmd(&message) =>
            {
                self.uninstall_legacy_pm(package_name.as_ref(), user)
            }
            Err(err) => Err(err),
        }
    }

    fn uninstall_legacy_pm(&mut self, package_name: &str, user: Option<&str>) -> Result<()> {
        let mut output = Vec::new();
        let mut stderr = Vec::new();
        let mut command = String::from("pm uninstall");
        if let Some(user) = user {
            command.push_str(" --user ");
            command.push_str(user);
        }
        command.push(' ');
        command.push_str(&shell_quote(package_name));

        let status = self.shell_command(&command, Some(&mut output), Some(&mut stderr))?;
        let combined = if stderr.is_empty() {
            String::from_utf8(output.clone())?
        } else {
            format!(
                "{}{}",
                String::from_utf8(output.clone())?,
                String::from_utf8(stderr.clone())?
            )
        };
        if status.is_none_or(|code| code == 0) && adb_command_output_is_success(&combined) {
            log::info!("Package {package_name} successfully uninstalled via legacy pm path");
            return Ok(());
        }

        Err(RustADBError::ADBRequestFailed(combined))
    }
}
