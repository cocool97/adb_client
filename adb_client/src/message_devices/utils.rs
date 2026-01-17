use bincode::config::{Configuration, Fixint, LittleEndian, NoLimit};

use serde::{Serialize, de::DeserializeOwned};

use crate::{Result, RustADBError};

const BINCODE_CONFIG: Configuration<LittleEndian, Fixint, NoLimit> = bincode::config::legacy();

pub(crate) fn serialize_to_vec<E: Serialize>(val: E) -> crate::Result<Vec<u8>> {
    bincode::serde::encode_to_vec(val, BINCODE_CONFIG).map_err(|_e| RustADBError::ConversionError)
}

pub(crate) fn deserialize_from_slice<D: DeserializeOwned>(data: &[u8]) -> Result<D> {
    let (response, _) = bincode::serde::decode_from_slice(data, BINCODE_CONFIG)
        .map_err(|_e| RustADBError::ConversionError)?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use crate::message_devices::{
        message_commands::{MessageSubcommand, SubcommandWithArg},
        utils::{deserialize_from_slice, serialize_to_vec},
    };

    #[test]
    fn test_bincode_serialize_deserialize_to_vec() {
        let quit_buffer = MessageSubcommand::Quit.with_arg(42u32);

        let serialized = serialize_to_vec(&quit_buffer).expect("cannot serialize struct");
        let deserialized: SubcommandWithArg =
            deserialize_from_slice(&serialized).expect("cannot deserialize struct");

        assert_eq!(
            quit_buffer, deserialized,
            "serialized data does not match deserialized"
        );
    }

    #[test]
    fn test_bincode_serialize_data_format() {
        let quit_buffer = MessageSubcommand::Quit.with_arg(42u32);

        let serialized = serialize_to_vec(&quit_buffer).expect("cannot serialize struct");

        // First 4 bytes should be the command value as u32
        assert_eq!(
            u32::from_le_bytes(
                serialized[0..(u32::BITS / 8) as usize]
                    .try_into()
                    .expect("invalid slice length")
            ),
            MessageSubcommand::Quit as u32,
            "invalid serialized command value"
        );

        // Next 4 bytes should be the argument value as u32
        assert_eq!(
            serialized[(u32::BITS / 8) as usize..(u32::BITS / 8) as usize + 4],
            42u32.to_le_bytes(),
            "invalid serialized argument value"
        );

        assert_eq!(serialized.len(), 8, "invalid serialized data length");
    }
}
