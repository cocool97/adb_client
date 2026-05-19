use std::{fs::File, path::Path};

use crate::{
    Result, RustADBError,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
    },
    utils::{
        ProgressReader, ProgressReporter, adb_command_output_is_success, check_extension_is_apk,
        shell_quote,
    },
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn install_with_progress(
        &mut self,
        apk_path: &dyn AsRef<Path>,
        user: Option<&str>,
        on_progress: Option<&mut dyn FnMut(u64, u64)>,
    ) -> Result<()> {
        check_extension_is_apk(apk_path)?;

        let mut progress = ProgressReporter::new(on_progress);
        self.install_legacy_pm(apk_path.as_ref(), user, progress.callback())
    }

    fn install_legacy_pm(
        &mut self,
        apk_path: &Path,
        user: Option<&str>,
        on_progress: Option<&mut dyn FnMut(u64, u64)>,
    ) -> Result<()> {
        let mut apk_file = File::open(apk_path)?;
        let file_size = apk_file.metadata()?.len();
        let apk_name = apk_path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| RustADBError::ADBRequestFailed("invalid APK filename".to_string()))?;
        let remote_apk = format!("/data/local/tmp/{apk_name}");

        let mut apk_reader = ProgressReader::new(&mut apk_file, file_size, on_progress);
        self.push(&mut apk_reader, &remote_apk)?;

        let mut output = Vec::new();
        let mut stderr = Vec::new();
        let mut command = String::from("pm install -r");
        if let Some(user) = user {
            command.push_str(" --user ");
            command.push_str(user);
        }
        command.push(' ');
        command.push_str(&shell_quote(&remote_apk));

        let status = self.shell_command(&command, Some(&mut output), Some(&mut stderr))?;
        let _ = self.shell_command(&format!("rm -f {}", shell_quote(&remote_apk)), None, None);
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
            log::info!(
                "APK file {} successfully installed via legacy pm path",
                apk_path.display()
            );
            return Ok(());
        }

        Err(RustADBError::ADBRequestFailed(combined))
    }
}
