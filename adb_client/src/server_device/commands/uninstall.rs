use std::io::Read;

use crate::{
    ADBDeviceExt, Result, RustADBError,
    models::{ADBCommand, ADBLocalCommand},
    server_device::ADBServerDevice,
    utils::{adb_command_output_is_success, adb_request_uses_missing_cmd, shell_quote},
};

impl ADBServerDevice {
    /// Uninstall a package from device
    pub fn uninstall(&mut self, package_name: &str, user: Option<&str>) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .send_adb_request(&ADBCommand::Local(ADBLocalCommand::Uninstall(
                package_name.to_string(),
                user.map(ToString::to_string),
            )))?;

        let mut data = [0; 1024];
        let read_amount = self.transport.get_raw_connection()?.read(&mut data)?;

        let uninstall_result = match &data[0..read_amount] {
            b"Success\n" => {
                log::info!("Package {package_name} successfully uninstalled");
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
                self.uninstall_legacy_pm(package_name, user)
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
