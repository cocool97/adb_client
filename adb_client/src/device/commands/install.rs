use std::fs::File;

use rand::Rng;

use crate::{
    device::{
        adb_message_device::ADBMessageDevice, ADBTransportMessage, MessageCommand, MessageWriter,
    },
    utils::check_extension_is_apk,
    ADBMessageTransport, Result,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn install<P: AsRef<std::path::Path>>(&mut self, apk_path: P) -> Result<()> {
        let mut apk_file = File::open(&apk_path)?;

        check_extension_is_apk(&apk_path)?;

        let file_size = apk_file.metadata()?.len();

        let mut rng = rand::thread_rng();

        let local_id = rng.gen();

        let message = ADBTransportMessage::new(
            MessageCommand::Open,
            local_id,
            0,
            format!("exec:cmd package 'install' -S {}\0", file_size)
                .as_bytes()
                .to_vec(),
        );
        self.get_transport_mut().write_message(message)?;

        let response = self.get_transport_mut().read_message()?;
        let remote_id = response.header().arg0();

        let transport = self.get_transport().clone();

        let mut writer = MessageWriter::new(transport, local_id, remote_id);

        std::io::copy(&mut apk_file, &mut writer)?;

        let final_status = self.get_transport_mut().read_message()?;

        match final_status.into_payload().as_slice() {
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
