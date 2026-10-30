use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use crate::{
    Result,
    message_devices::{
        adb_message_device::ADBMessageDevice, adb_message_transport::ADBMessageTransport,
        commands::utils::MessageWriter,
    },
    models::ADBLocalCommand,
    utils::check_extension_is_apk,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn install(&self, apk_path: &dyn AsRef<Path>, user: Option<&str>) -> Result<()> {
        let mut apk_file = File::open(apk_path)?;

        check_extension_is_apk(apk_path)?;

        let file_size = apk_file.metadata()?.len();

        let mut session = self.open_session(&ADBLocalCommand::Install(
            file_size,
            user.map(ToString::to_string),
        ))?;

        {
            // Read data from apk_file and write it to the underlying session, in messages as large as possible
            let mut buffer = vec![0; session.max_payload_size()].into_boxed_slice();
            let mut writer = MessageWriter::new(&mut session);
            loop {
                let size = apk_file.read(&mut buffer)?;
                if size == 0 {
                    break;
                }
                writer.write_all(&buffer[..size])?;
            }
        }

        match session.recv_until_close()?.as_slice() {
            b"Success\n" => {
                log::info!(
                    "APK file {} successfully installed",
                    apk_path.as_ref().display()
                );
                Ok(())
            }
            d => Err(crate::RustADBError::ADBRequestFailed(String::from_utf8(
                d.to_vec(),
            )?)),
        }
    }
}
