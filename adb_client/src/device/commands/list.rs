use crate::{
    ADBListItem, ADBListItemType, ADBMessageTransport, Result, RustADBError,
    device::{
        ADBTransportMessage, MessageCommand, MessageSubcommand,
        adb_message_device::{ADBMessageDevice, bincode_serialize_to_vec},
    },
};
use byteorder::ByteOrder;
use byteorder::LittleEndian;
use std::str;

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    /// List the entries in the given directory on the device.
    /// note: path uses internal file paths, so Documents is at /storage/emulated/0/Documents
    pub(crate) fn list<A: AsRef<str>>(&mut self, path: A) -> Result<Vec<ADBListItem>> {
        let session = self.begin_synchronization()?;

        let output = self.handle_list(path, session.local_id(), session.remote_id());

        self.end_transaction(session)?;
        output
    }

    /// Request amount of bytes from transport, potentially across payloads
    ///
    /// This automatically request a new payload by sending back "Okay" and waiting for the next payload
    /// It reads the request bytes across the existing payload, and if there is not enough bytes left,
    /// reads the rest from the next payload
    ///
    ///   Current index
    /// ┼───────────────┼   Requested
    ///                 ┌─────────────┐
    /// ┌───────────────┼───────┐     │
    /// └───────────────────────┘
    ///     Current             └─────┘
    ///     payload          Wanted in
    ///                      Next payload
    fn read_bytes_from_transport(
        requested_bytes: usize,
        current_index: &mut usize,
        transport: &mut T,
        payload: &mut Vec<u8>,
        local_id: u32,
        remote_id: u32,
    ) -> Result<Vec<u8>> {
        if *current_index + requested_bytes <= payload.len() {
            // if there is enough bytes in this payload
            // Copy from existing payload
            let slice = &payload[*current_index..*current_index + requested_bytes];
            *current_index += requested_bytes;
            Ok(slice.to_vec())
        } else {
            // Read the rest of the existing payload, then continue with the next message
            let mut slice = Vec::new();
            let bytes_read_from_existing_payload = payload.len() - *current_index;
            slice.extend_from_slice(
                &payload[*current_index..*current_index + bytes_read_from_existing_payload],
            );

            // Request the next message
            let send_message =
                ADBTransportMessage::try_new(MessageCommand::Okay, local_id, remote_id, &[])?;
            transport.write_message(send_message)?;
            // Read the new message
            *payload = transport.read_message()?.into_payload();
            let bytes_read_from_new_payload = requested_bytes - bytes_read_from_existing_payload;
            slice.extend_from_slice(&payload[..bytes_read_from_new_payload]);
            *current_index = bytes_read_from_new_payload;
            Ok(slice)
        }
    }

    fn handle_list<A: AsRef<str>>(
        &mut self,
        path: A,
        local_id: u32,
        remote_id: u32,
    ) -> Result<Vec<ADBListItem>> {
        // TODO: use LIS2 to support files over 2.14 GB in size.
        // SEE: https://github.com/cstyan/adbDocumentation?tab=readme-ov-file#adb-list
        {
            let mut len_buf = Vec::from([0_u8; 4]);
            LittleEndian::write_u32(&mut len_buf, u32::try_from(path.as_ref().len())?);

            let subcommand_data = MessageSubcommand::List;

            let mut serialized_message = bincode_serialize_to_vec(subcommand_data)
                .map_err(|_e| RustADBError::ConversionError)?;

            serialized_message.append(&mut len_buf);
            let mut path_bytes: Vec<u8> = Vec::from(path.as_ref().as_bytes());
            serialized_message.append(&mut path_bytes);

            let message = ADBTransportMessage::try_new(
                MessageCommand::Write,
                local_id,
                remote_id,
                &serialized_message,
            )?;
            self.send_and_expect_okay(message)?;
        }

        let mut list_items = Vec::new();

        let transport = self.get_transport_mut();
        let mut payload = transport.read_message()?.into_payload();
        let mut current_index = 0;
        loop {
            // Loop though the response for all the entries
            const STATUS_CODE_LENGTH_IN_BYTES: usize = 4;
            let status_code = Self::read_bytes_from_transport(
                STATUS_CODE_LENGTH_IN_BYTES,
                &mut current_index,
                transport,
                &mut payload,
                local_id,
                remote_id,
            )?;
            match str::from_utf8(&status_code)? {
                "DENT" => {
                    // Read the file mode, size, mod time and name length in one go, since all their sizes are predictable
                    const U32_SIZE_IN_BYTES: usize = 4;
                    const SIZE_OF_METADATA: usize = U32_SIZE_IN_BYTES * 4;
                    let metadata = Self::read_bytes_from_transport(
                        SIZE_OF_METADATA,
                        &mut current_index,
                        transport,
                        &mut payload,
                        local_id,
                        remote_id,
                    )?;
                    let mode = metadata[..U32_SIZE_IN_BYTES].to_vec();
                    let size = metadata[U32_SIZE_IN_BYTES..2 * U32_SIZE_IN_BYTES].to_vec();
                    let time = metadata[2 * U32_SIZE_IN_BYTES..3 * U32_SIZE_IN_BYTES].to_vec();
                    let name_len = metadata[3 * U32_SIZE_IN_BYTES..4 * U32_SIZE_IN_BYTES].to_vec();

                    let mode = LittleEndian::read_u32(&mode);
                    let size = LittleEndian::read_u32(&size);
                    let time = LittleEndian::read_u32(&time);
                    let name_len = LittleEndian::read_u32(&name_len) as usize;
                    // Read the file name, since it requires the length from the name_len
                    let name_buf = Self::read_bytes_from_transport(
                        name_len,
                        &mut current_index,
                        transport,
                        &mut payload,
                        local_id,
                        remote_id,
                    )?;
                    let name = String::from_utf8(name_buf)?;

                    // First 9 bits are the file permissions
                    let permissions = mode & 0b1_1111_1111;
                    // Bits 14 to 16 are the file type
                    let item_type = match (mode >> 13) & 0b111 {
                        0b010 => ADBListItemType::Directory,
                        0b100 => ADBListItemType::File,
                        0b101 => ADBListItemType::Symlink,
                        type_code => return Err(RustADBError::UnknownFileMode(type_code)),
                    };
                    let entry = ADBListItem {
                        name,
                        time,
                        permissions,
                        size,
                        item_type,
                    };
                    list_items.push(entry);
                }
                "DONE" => {
                    return Ok(list_items);
                }
                x => log::error!("Got an unknown response {x}"),
            }
        }
    }
}
