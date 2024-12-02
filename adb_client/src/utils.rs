use std::{ffi::OsStr, path::Path};

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

    Ok(())
}
