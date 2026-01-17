use bincode::config::{Configuration, Fixint, LittleEndian, NoLimit};

use serde::{Serialize, de::DeserializeOwned};

use crate::{Result, RustADBError};

const BINCODE_CONFIG: Configuration<LittleEndian, Fixint, NoLimit> = bincode::config::legacy();

pub(crate) fn bincode_serialize_to_vec<E: Serialize>(val: E) -> crate::Result<Vec<u8>> {
    bincode::serde::encode_to_vec(val, BINCODE_CONFIG).map_err(|_e| RustADBError::ConversionError)
}

pub(crate) fn bincode_deserialize_from_slice<D: DeserializeOwned>(data: &[u8]) -> Result<D> {
    let (response, _) = bincode::serde::decode_from_slice(data, BINCODE_CONFIG)
        .map_err(|_e| RustADBError::ConversionError)?;

    Ok(response)
}
