use std::{ffi::OsStr, path::Path};

use crate::{Result, RustADBError};

pub fn u32_from_le(value: &[u8]) -> Result<u32> {
    Ok(u32::from_le_bytes(
        value
            .try_into()
            .map_err(|_| RustADBError::ConversionError)?,
    ))
}

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
