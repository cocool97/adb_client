use std::{
    ffi::OsStr,
    io::Read,
    path::{Path, PathBuf},
};

use crate::{Result, RustADBError};

pub fn check_extension_is_apk<P: AsRef<Path>>(path: P) -> Result<()> {
    if let Some(extension) = path.as_ref().extension()
        && ![OsStr::new("apk")].contains(&extension)
    {
        return Err(RustADBError::WrongFileExtension(format!(
            "{} is not an APK file",
            extension.to_string_lossy()
        )));
    }

    log::debug!("Given file is an APK");

    Ok(())
}

/// Get the default path to the ADB key file.
/// First checks for the presence of the environment variable `ANDROID_USER_HOME`, defaulting to the user's home directory.
pub fn get_default_adb_key_path() -> Result<PathBuf> {
    let android_user_home = std::env::var("ANDROID_USER_HOME")
        .ok()
        .map(|android_user_home| PathBuf::from(android_user_home).join("android"));
    let default_dot_android = std::env::home_dir().map(|home| home.join(".android"));

    Ok(android_user_home
        .or(default_dot_android)
        .ok_or(RustADBError::NoHomeDirectory)?
        .join("adbkey"))
}

pub(crate) fn adb_request_uses_missing_cmd(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("cmd: not found")
        || lower.contains("cmd: inaccessible or not found")
        || lower.contains("/system/bin/sh: cmd:")
}

pub(crate) fn adb_command_output_is_success(output: &str) -> bool {
    output.lines().any(|line| line.trim() == "Success")
}

pub(crate) fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

pub(crate) struct ProgressReader<'a, R: Read> {
    inner: R,
    uploaded: u64,
    total: u64,
    on_progress: Option<&'a mut dyn FnMut(u64, u64)>,
}

impl<'a, R: Read> ProgressReader<'a, R> {
    pub fn new(inner: R, total: u64, on_progress: Option<&'a mut dyn FnMut(u64, u64)>) -> Self {
        Self {
            inner,
            uploaded: 0,
            total,
            on_progress,
        }
    }
}

pub(crate) struct ProgressReporter<'a> {
    callback: Option<&'a mut dyn FnMut(u64, u64)>,
}

impl<'a> ProgressReporter<'a> {
    pub fn new(callback: Option<&'a mut dyn FnMut(u64, u64)>) -> Self {
        Self { callback }
    }

    pub fn callback(&mut self) -> Option<&mut dyn FnMut(u64, u64)> {
        match self.callback.as_mut() {
            Some(callback) => Some(&mut **callback),
            None => None,
        }
    }
}

impl<R: Read> Read for ProgressReader<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buf)?;
        if read == 0 {
            return Ok(0);
        }

        self.uploaded = self.uploaded.saturating_add(read as u64);
        if let Some(on_progress) = self.on_progress.as_mut() {
            on_progress(self.uploaded, self.total);
        }
        Ok(read)
    }
}

#[cfg(test)]
mod tests {
    use super::{adb_command_output_is_success, adb_request_uses_missing_cmd, shell_quote};

    #[test]
    fn detects_missing_cmd_errors() {
        assert!(adb_request_uses_missing_cmd(
            "/system/bin/sh: cmd: not found"
        ));
        assert!(adb_request_uses_missing_cmd(
            "cmd: inaccessible or not found"
        ));
        assert!(!adb_request_uses_missing_cmd(
            "Failure [DELETE_FAILED_INTERNAL_ERROR]"
        ));
    }

    #[test]
    fn detects_success_from_any_output_line() {
        assert!(adb_command_output_is_success("Success"));
        assert!(adb_command_output_is_success(
            "pkg: /data/local/tmp/app-release.apk\nSuccess"
        ));
        assert!(!adb_command_output_is_success(
            "Failure [INSTALL_FAILED_UPDATE_INCOMPATIBLE]"
        ));
    }

    #[test]
    fn shell_quote_handles_empty_and_single_quotes() {
        assert_eq!(shell_quote("plain"), "'plain'");
        assert_eq!(shell_quote("a'b"), "'a'\"'\"'b'");
        assert_eq!(shell_quote(""), "''");
    }
}
