use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::{Result, RustADBError};

pub fn check_extension_is_apk<P: AsRef<Path>>(path: P) -> Result<()> {
    if let Some(extension) = path.as_ref().extension() {
        if ![OsStr::new("apk")].contains(&extension) {
            return Err(RustADBError::WrongFileExtension(format!(
                "{} is not an APK file",
                extension.to_string_lossy()
            )));
        }
    }

    log::debug!("Given file is an APK");

    Ok(())
}

pub fn get_default_adb_key_path() -> Result<PathBuf> {
    homedir::my_home()
        .ok()
        .flatten()
        .map(|home| home.join(".android").join("adbkey"))
        .ok_or(RustADBError::NoHomeDirectory)
}
