use crate::{Result, RustADBError};

pub fn u32_from_le(value: &[u8]) -> Result<u32> {
    Ok(u32::from_le_bytes(
        value
            .try_into()
            .map_err(|_| RustADBError::ConversionError)?,
    ))
}
